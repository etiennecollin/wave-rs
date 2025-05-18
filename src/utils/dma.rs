use embassy_stm32::{
    pac::timer::vals::Ccds,
    time::Hertz,
    timer::{
        low_level::{CountingMode, Timer},
        AdvancedInstance4Channel, Channel,
    },
    Peri,
};

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
