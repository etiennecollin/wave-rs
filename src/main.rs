#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{exti::ExtiInput, gpio::Pull, rng::Rng, Config};
use wave_rs::{
    keyboard::{dma::configure_dma_scan, mouse::mouse_writer_task, scan::keyboard_scan_task},
    usb::{
        ethernet::{init_ethernet, usb_ethernet_task},
        hid::{hid_keyboard_reader_task, init_hid_keyboard, init_hid_mouse},
        serial::{init_serial, usb_serial_task},
        usb_device::{init_usb, usb_task},
    },
    web::{
        network_stack::{init_network_stack, network_stack_task},
        web_server::web_server_task,
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
        config.rcc.mux.otghssel = mux::Otghssel::PLL1_P;

        // Setup low speed clock
        config.rcc.ls = LsConfig::default_lsi();
        config.enable_debug_during_sleep = false;

        // Setup RNG
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // See Note
        config.rcc.mux.rngsel = mux::Rngsel::HSI48;

        // Note:
        // Section 48.3.6: RNG Clocking
        // When the CED bit in the RNG_CR register is set to 0 (error detection
        // enabled), the RNG clock frequency before the internal divider must
        // be higher than the AHB clock frequency divided by 32, otherwise the
        // clock checker always flags a clock error (CECS = 1 in the RNG_SR
        // register).
    }
    let p = embassy_stm32::init(config);

    // =========================================================================
    // Configure important peripherals
    // =========================================================================
    // Configure the RNG
    let mut rng = Rng::new(p.RNG, Irqs);

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
    // Initialize USB Peripherals
    // =========================================================================
    defmt::info!("Creating USB classes...");

    // Serial
    let class_serial = init_serial(&mut builder).await;

    // HID
    let (hid_keyboard_reader, hid_keyboard_writer) = init_hid_keyboard(&mut builder).await;
    let hid_mouse_writer = init_hid_mouse(&mut builder).await;

    // Network
    let (eth_runner, eth_device) = init_ethernet(&mut builder).await;
    let (stack, stack_runner) = init_network_stack(eth_device, &mut rng).await;

    // Build the usb device
    defmt::info!("Building USB device...");
    let usb = builder.build();

    // =========================================================================
    // Initialize GPDMA
    // =========================================================================
    let (mut write_ring_buffer, mut read_ring_buffer) =
        configure_dma_scan(p.GPDMA1_CH0.into(), p.GPDMA1_CH1.into());

    // Start the DMA
    write_ring_buffer.start();
    read_ring_buffer.start();

    // =========================================================================
    // Spawn USB tasks
    // =========================================================================
    // Spawn tasks
    defmt::info!("Spawning USB tasks...");
    spawner.spawn(usb_task(usb)).unwrap();

    // Serial
    spawner.spawn(usb_serial_task(class_serial)).unwrap();

    // HID
    spawner
        .spawn(hid_keyboard_reader_task(hid_keyboard_reader))
        .unwrap();
    spawner
        .spawn(keyboard_scan_task(
            hid_keyboard_writer,
            write_ring_buffer,
            read_ring_buffer,
        ))
        .unwrap();
    // spawner.spawn(mouse_writer_task(hid_mouse_writer)).unwrap();

    // Network stack
    spawner.spawn(usb_ethernet_task(eth_runner)).unwrap();
    spawner.spawn(network_stack_task(stack_runner)).unwrap();
    spawner.spawn(web_server_task(stack)).unwrap();
}
