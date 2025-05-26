use embassy_stm32::{
    dma::{
        linked_list::LinearItem, AnyChannel, LinkedListTransfer, Priority, ReadableRingBuffer,
        TransferOptions, WritableRingBuffer,
    },
    pac::{self, timer::vals::Ccds},
    time::Hertz,
    timer::{
        low_level::{CountingMode, Timer},
        AdvancedInstance4Channel, Channel,
    },
    Peri,
};
use static_cell::StaticCell;

use crate::config::{MATRIX_COLUMNS_GPIO_PORT, MATRIX_COLUMNS_NUMBER, MATRIX_ROWS_GPIO_PORT};

/// Number of items in the GPDMA linked list.
pub const LINKED_LIST_LENGTH: usize = 2;
/// Size of the GPDMA linked list items.
pub type LinkedListWord = u32;

pub fn configure_dma_scan(
    dma_write_channel: Peri<'static, AnyChannel>,
    dma_read_channel: Peri<'static, AnyChannel>,
) -> (
    WritableRingBuffer<'static, LinkedListWord, LINKED_LIST_LENGTH>,
    ReadableRingBuffer<'static, LinkedListWord, LINKED_LIST_LENGTH>,
) {
    // The write buffer is used to write to the GPIO registers
    // This will enable each GPIO column pin one by one using a bit mask
    static DMA_WRITE_BUFFER: StaticCell<[LinkedListWord; MATRIX_COLUMNS_NUMBER * 2]> =
        StaticCell::new();
    let mut write_bit_masks = [0; MATRIX_COLUMNS_NUMBER * 2];

    // Create a mask which will reset all column pins.
    // We will mask out of this mask the pin to turn on.
    let reset_mask = (1 << MATRIX_COLUMNS_NUMBER) - 1;

    // Fill the write buffer with the bit masks.
    // Bytes [31, 16] dictate which pins to reset.
    // Bytes [15, 0] dictate which pin to set.
    write_bit_masks.iter_mut().enumerate().for_each(|(i, x)| {
        let pos = 1 << (i % MATRIX_COLUMNS_NUMBER) as u32;
        *x = (reset_mask ^ pos) << 16 | pos
    });

    // Save the write buffer to static memory
    let write_buffer = DMA_WRITE_BUFFER.init(write_bit_masks);

    // The read buffer is used to read from the GPIO registers.
    // We use double buffering to prevent race conditions between
    // the reading and writing of the GPIO registers.
    static DMA_READ_BUFFER: StaticCell<[LinkedListWord; MATRIX_COLUMNS_NUMBER * 2]> =
        StaticCell::new();
    let read_buffer = DMA_READ_BUFFER.init([0; MATRIX_COLUMNS_NUMBER * 2]);

    // Set DMA options
    let mut transfer_options = TransferOptions::default();
    transfer_options.priority = Priority::VeryHigh;
    transfer_options.half_transfer_ir = false;
    transfer_options.complete_transfer_ir = true;

    // Create the linked list transfer tables
    let mut write_table = unsafe {
        WritableRingBuffer::<_, LINKED_LIST_LENGTH>::new_ping_pong_table(
            42, // Trigger on tim1_cc1_dma (see STM32U5 reference manual p.688).
            MATRIX_COLUMNS_GPIO_PORT.bsrr().as_ptr() as _,
            write_buffer,
        )
    };
    let mut read_table = unsafe {
        ReadableRingBuffer::<_, LINKED_LIST_LENGTH>::new_ping_pong_table(
            43, // Trigger on tim1_cc2_dma (see STM32U5 reference manual p.688).
            MATRIX_ROWS_GPIO_PORT.idr().as_ptr() as _,
            read_buffer,
        )
    };

    // Override the default settings for the linked list transfer tables
    write_table.items.iter_mut().for_each(|item| {
        item.tr2.set_swreq(pac::gpdma::vals::Swreq::HARDWARE);
        // item.tr2.set_trigm(pac::gpdma::vals::Trigm::BLOCK);
        // item.tr2.set_trigpol(pac::gpdma::vals::Trigpol::RISING_EDGE);
        item.tr2.set_tcem(pac::gpdma::vals::Tcem::EACH_BLOCK);
        // item.tr2.set_trigsel(0);
    });
    read_table.items.iter_mut().for_each(|item| {
        item.tr2.set_swreq(pac::gpdma::vals::Swreq::HARDWARE);
        // item.tr2.set_trigm(pac::gpdma::vals::Trigm::BLOCK);
        // item.tr2.set_trigpol(pac::gpdma::vals::Trigpol::RISING_EDGE);
        item.tr2.set_tcem(pac::gpdma::vals::Tcem::EACH_BLOCK);
        // item.tr2.set_trigsel(0);
    });

    // Create the GPDMA ring-buffers
    let write_ring_buffer = WritableRingBuffer::new_with_table(
        dma_write_channel,
        write_buffer,
        transfer_options,
        write_table,
    );
    let read_ring_buffer = ReadableRingBuffer::new_with_table(
        dma_read_channel,
        read_buffer,
        transfer_options,
        read_table,
    );

    (write_ring_buffer, read_ring_buffer)
}

pub struct DmaTimer<T: AdvancedInstance4Channel> {
    timer: Timer<'static, T>,
}

impl<T: AdvancedInstance4Channel> DmaTimer<T> {
    pub fn new(timer: Peri<'static, T>) -> Self {
        Self {
            timer: Timer::new(timer),
        }
    }

    /// Get max compare value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_compare_value(&self) -> u32 {
        self.timer.get_max_compare_value()
    }

    /// Configures a timer with DMA interrupts activated. The counter counts up
    /// to `compare_value_max` with a frequency of `frequency` and then resets
    /// to 0.
    ///
    /// The timer is configured to generate DMA interrupts on channel 1 and 2
    /// when `compare_value_ch_1` and `compare_value_ch_2` are reached
    /// respectively. The timer is configured to count up in edge-aligned mode.
    pub fn configure(
        &mut self,
        frequency: Hertz,
        compare_value_ch_1: u32,
        compare_value_ch_2: u32,
        compare_value_max: u32,
    ) {
        // Make sure the compare values make sense
        {
            assert!(compare_value_max <= u16::MAX as u32);
            assert!(compare_value_ch_1 <= compare_value_max);
            assert!(compare_value_ch_2 <= compare_value_max);
        }

        // General configuration
        self.timer.set_autoreload_preload(true);
        self.timer.set_max_compare_value(compare_value_max);
        self.timer
            .set_frequency(frequency * MATRIX_COLUMNS_NUMBER as u32);
        self.timer.set_counting_mode(CountingMode::EdgeAlignedUp);

        // Enable interrupts on compare match
        self.timer
            .set_compare_value(Channel::Ch1, compare_value_ch_1);
        self.timer
            .set_compare_value(Channel::Ch2, compare_value_ch_2);

        // Enable DMA on compare match
        self.timer.enable_update_dma(true);
        self.timer.set_cc_dma_selection(Ccds::ON_COMPARE);
        self.timer.set_cc_dma_enable_state(Channel::Ch1, true);
        self.timer.set_cc_dma_enable_state(Channel::Ch2, true);
    }

    /// Start the timer.
    pub fn start(&mut self) {
        self.timer.start();
    }
}
