#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    rng::Rng,
    Config,
};
use wave_rs::{
    blinky::{blinky, button_listen},
    usb::{
        ethernet::{init_ethernet, usb_ethernet_task},
        network_stack::{init_network_stack, network_stack_task, web_server_task},
        serial::{init_serial, usb_serial_task},
        serial_logger::{init_serial_logger, usb_serial_logger_task},
        usb_device::{init_usb, usb_task},
    },
    Irqs,
};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // ========================================================================
    // Initialization of STM32
    // =========================================================================
    defmt::info!("Configuring STM32 clocks...");
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::time::Hertz;
        config.rcc.hse = Some(Hse {
            freq: Hertz(16_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,   // HSE / 2 = 8MHz
            mul: PllMul::MUL60,        // 8MHz * 60 = 480MHz
            divr: Some(PllDiv::DIV3),  // 480MHz / 3 = 160MHz (sys_ck)
            divq: Some(PllDiv::DIV10), // 480MHz / 10 = 48MHz (USB)
            divp: Some(PllDiv::DIV15), // 480MHz / 15 = 32MHz (USBOTG)
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.voltage_range = VoltageScale::RANGE1;
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // See Note

        config.rcc.mux.otghssel = mux::Otghssel::PLL1_P;
        // config.rcc.mux.rngsel = mux::Rngsel::HSI48;

        // Note:
        // 48.3.6 RNG Clocking
        // When the CED bit in the RNG_CR register is set to 0 (error detection enabled), the RNG clock frequency before the internal divider must be higher than the AHB clock frequency divided by 32, otherwise the clock checker always flags a clock error (CECS = 1 in the RNG_SR register).
    }
    let p = embassy_stm32::init(config);

    // =========================================================================
    // Configure important peripherals
    // =========================================================================
    let mut rng = Rng::new(p.RNG, Irqs);

    // =========================================================================
    // Blinky
    // =========================================================================
    defmt::info!("Starting blinky tasks...");
    // Prepare peripherals for blinky
    let leds: [Output; 3] = [
        Output::new(p.PG2, Level::Low, Speed::Low), // red
        Output::new(p.PB7, Level::Low, Speed::Low), // blue
        Output::new(p.PC7, Level::Low, Speed::Low), // green
    ];
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Down);

    spawner.spawn(button_listen(button)).unwrap();
    spawner.spawn(blinky(leds)).unwrap();

    // =========================================================================
    // USB Builder
    // =========================================================================
    defmt::info!("Initializing USB...");
    let mut builder = init_usb(p.USB_OTG_HS, p.PA12, p.PA11).await;

    // =========================================================================
    // Setup DFU
    // =========================================================================
    // let flash = Flash::new_blocking(p.FLASH);
    // let flash = Mutex::new(RefCell::new(flash));
    //
    // let config = FirmwareUpdaterConfig::from_linkerfile_blocking(&flash, &flash);
    // let mut magic = AlignedBuffer([0; WRITE_SIZE]);
    // let mut firmware_state = BlockingFirmwareState::from_config(config, &mut magic.0);
    // firmware_state.mark_booted().expect("Failed to mark booted");
    // let mut state = Control::new(firmware_state, DfuAttributes::CAN_DOWNLOAD);
    // usb_dfu::<_, _, ResetImmediate>(&mut builder, &mut state, Duration::from_millis(2500));

    // =========================================================================
    // USB Peripherals
    // =========================================================================
    defmt::info!("Creating USB classes...");
    let class_serial = init_serial(&mut builder).await;
    #[cfg(feature = "log-serial")]
    let class_serial_logger = init_serial_logger(&mut builder).await;
    let (eth_runner, eth_device) = init_ethernet(&mut builder).await;
    let (stack, stack_runner) = init_network_stack(eth_device, &mut rng).await;

    // Build the usb device
    defmt::info!("Building USB device...");
    let usb = builder.build();

    // Spawn tasks
    defmt::info!("Spawning USB tasks...");
    spawner.spawn(usb_task(usb)).unwrap();
    spawner.spawn(usb_serial_task(class_serial)).unwrap();
    #[cfg(feature = "log-serial")]
    spawner
        .spawn(usb_serial_logger_task(class_serial_logger))
        .unwrap();
    spawner.spawn(usb_ethernet_task(eth_runner)).unwrap();
    spawner.spawn(network_stack_task(stack_runner)).unwrap();
    spawner.spawn(web_server_task(stack)).unwrap();
}
