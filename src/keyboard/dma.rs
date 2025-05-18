use embassy_stm32::{
    dma::{AnyChannel, Priority, ReadableRingBuffer, Request, TransferOptions, WritableRingBuffer},
    pac::timer::vals::Ccds,
    time::Hertz,
    timer::{
        low_level::{CountingMode, Timer},
        AdvancedInstance4Channel, Channel,
    },
    Peri,
};
use static_cell::StaticCell;

use crate::config::{GPIO_PORT_COLUMNS, GPIO_PORT_ROWS, NUMBER_COLUMNS, NUMBER_ROWS};

pub fn configure_dma_scan(
    dma_write_channel: Peri<'static, AnyChannel>,
    dma_read_channel: Peri<'static, AnyChannel>,
) -> (
    WritableRingBuffer<'static, u16>,
    ReadableRingBuffer<'static, u16>,
) {
    // The write buffer is used to write to the GPIO registers
    // This is will enable each GPIO column pin one by one using a "bit mask"
    static DMA_WRITE_BUFFER: StaticCell<[u16; NUMBER_COLUMNS * 2]> = StaticCell::new();
    let mut write_bit_mask = [0u16; NUMBER_COLUMNS * 2];
    write_bit_mask.iter_mut().enumerate().for_each(|(i, x)| {
        *x = 1 << (i % NUMBER_COLUMNS) as u16;
    });
    let write_buffer = DMA_WRITE_BUFFER.init(write_bit_mask);

    // The read buffer is used to read from the GPIO registers.
    // We use double buffering to prevent race conditions between
    // the reading and writing of the GPIO registers.
    static DMA_READ_BUFFER: StaticCell<[u16; NUMBER_ROWS * 2]> = StaticCell::new();
    let read_buffer = DMA_READ_BUFFER.init([0u16; NUMBER_ROWS * 2]);

    // Set DMA options
    let mut transfer_options = TransferOptions::default();
    transfer_options.priority = Priority::VeryHigh;
    transfer_options.half_transfer_ir = false;
    transfer_options.complete_transfer_ir = true;

    let write_ring_buffer = unsafe {
        WritableRingBuffer::new(
            dma_write_channel,
            Request::default(),
            GPIO_PORT_COLUMNS.idr().as_ptr() as *mut u16,
            write_buffer,
            transfer_options,
        )
    };

    let read_ring_buffer = unsafe {
        ReadableRingBuffer::new(
            dma_read_channel,
            Request::default(),
            GPIO_PORT_ROWS.odr().as_ptr() as *mut u16,
            read_buffer,
            transfer_options,
        )
    };

    // TODO: Before starting the DMA, we need to set up the trigger to make
    // sure the DMA channels do not race
    // ch.tr2().write(|w| {
    //     w.set_trigsel(22); // Trigger on gpdma1_ch0_tc (based on STM32U5 reference manual p.692).
    //     w.set_swreq(pac::gpdma::vals::Swreq::HARDWARE);
    //     w.set_trigm(pac::gpdma::vals::Trigm::LINKED_LIST_ITEM);
    //     w.set_trigpol(pac::gpdma::vals::Trigpol::RISING_EDGE);
    //     w.set_tcem(pac::gpdma::vals::Tcem::EACH_LINKED_LIST_ITEM);
    // });

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

    /// The counter of the timer starts at 0 and is incremented until `max_compare_value()`
    /// is reached. Then it resets to 0. The compare value of each channel is set to trigger
    /// the DMA transfer at different points in the counter cycle.
    pub fn configure(
        &mut self,
        frequency: Hertz,
        compare_value_write: u32,
        compare_value_read: u32,
    ) {
        // Make sure the compare values make sense
        {
            let max = self.max_compare_value();
            assert!(compare_value_write < max);
            assert!(compare_value_read < max);
            assert!(compare_value_write < compare_value_read);
        }

        // General configuration
        self.timer.set_frequency(frequency);
        self.timer.set_counting_mode(CountingMode::EdgeAlignedUp);
        self.timer.enable_update_dma(true);
        self.timer.set_cc_dma_selection(Ccds::ON_COMPARE);
        self.timer.set_cc_dma_enable_state(Channel::Ch1, true);
        self.timer.set_cc_dma_enable_state(Channel::Ch2, true);

        // Channel 1 is used to trigger DMA writes to the GPIO registers.
        // Channel 2 is used to trigger DMA reads from the GPIO registers.
        // This happens when the compare value is reached.
        self.timer
            .set_compare_value(Channel::Ch1, compare_value_write);
        self.timer
            .set_compare_value(Channel::Ch2, compare_value_read);

        // Start the timer
        self.timer.start();
    }
}
