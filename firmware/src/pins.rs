use stm32f1xx_hal::{
    gpio::*,
    pac::{GPIOA, GPIOB},
};

pub type ColMode = Input<PullUp>;
pub type RowMode = Output<PushPull>;

pub type ErasedCol = ErasedPin<ColMode>;
pub type ErasedRow = ErasedPin<RowMode>;

// TODO: correct these pin assignments once design is done
pub type Col0 = PB0<ColMode>;
pub type Col1 = PB1<ColMode>;
pub type Col2 = PB2<ColMode>;
pub type Col3 = PB5<ColMode>;
pub type Col4 = PB6<ColMode>;
pub type Col5 = PB7<ColMode>;
pub type Col6 = PB8<ColMode>;
pub type Col7 = PB9<ColMode>;
pub type Col8 = PB10<ColMode>;
pub type Col9 = PB11<ColMode>;
pub type Col10 = PB12<ColMode>;
pub type Col11 = PB13<ColMode>;
pub type Col12 = PB14<ColMode>;
pub type Col13 = PB15<ColMode>;

pub type Row0 = PA0<RowMode>;
pub type Row1 = PA1<RowMode>;
pub type Row2 = PA2<RowMode>;
pub type Row3 = PA3<RowMode>;
pub type Row4 = PA4<RowMode>;
pub type Row5 = PA5<RowMode>;

// TODO: correct LED pin(s)
pub type CapsLock = PA6<Output<PushPull>>;

pub type UsbDm = PA11;
pub type UsbDp = PA12;

pub struct Pins {
    pub col0: Col0,
    pub col1: Col1,
    pub col2: Col2,
    pub col3: Col3,
    pub col4: Col4,
    pub col5: Col5,
    pub col6: Col6,
    pub col7: Col7,
    pub col8: Col8,
    pub col9: Col9,
    pub col10: Col10,
    pub col11: Col11,
    pub col12: Col12,
    pub col13: Col13,

    pub row0: Row0,
    pub row1: Row1,
    pub row2: Row2,
    pub row3: Row3,
    pub row4: Row4,
    pub row5: Row5,

    pub caps_lock: CapsLock,

    pub usb_dm: UsbDm,
    pub usb_dp: UsbDp,
}

/// Abstraction helper to split out all the pins needed in their correct states.
///
/// Side note: It's weird how this HAL needs registers for configuring pins.
pub fn pins(gpioa: GPIOA, gpiob: GPIOB) -> Pins {
    let mut gpioa = gpioa.split();
    let mut gpiob = gpiob.split();

    Pins {
        col0: gpiob.pb0.into_pull_up_input(&mut gpiob.crl),
        col1: gpiob.pb1.into_pull_up_input(&mut gpiob.crl),
        col2: gpiob.pb2.into_pull_up_input(&mut gpiob.crl),
        col3: gpiob.pb5.into_pull_up_input(&mut gpiob.crl),
        col4: gpiob.pb6.into_pull_up_input(&mut gpiob.crl),
        col5: gpiob.pb7.into_pull_up_input(&mut gpiob.crl),
        col6: gpiob.pb8.into_pull_up_input(&mut gpiob.crh),
        col7: gpiob.pb9.into_pull_up_input(&mut gpiob.crh),
        col8: gpiob.pb10.into_pull_up_input(&mut gpiob.crh),
        col9: gpiob.pb11.into_pull_up_input(&mut gpiob.crh),
        col10: gpiob.pb12.into_pull_up_input(&mut gpiob.crh),
        col11: gpiob.pb13.into_pull_up_input(&mut gpiob.crh),
        col12: gpiob.pb14.into_pull_up_input(&mut gpiob.crh),
        col13: gpiob.pb15.into_pull_up_input(&mut gpiob.crh),

        row0: gpioa.pa0.into_push_pull_output(&mut gpioa.crl),
        row1: gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
        row2: gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
        row3: gpioa.pa3.into_push_pull_output(&mut gpioa.crl),
        row4: gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
        row5: gpioa.pa5.into_push_pull_output(&mut gpioa.crl),

        caps_lock: gpioa.pa6.into_push_pull_output(&mut gpioa.crl),

        usb_dm: gpioa.pa11,
        usb_dp: gpioa.pa12,
    }
}
