#[doc = r" The RTIC application module"] pub mod app
{
    #[doc =
    r" Always include the device crate which contains the vector table"] use
    crate :: hal :: pac as
    you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml; use super
    :: * ; #[doc = r" User code from within the module"]
    #[doc = r" User code end"] #[doc = " User provided init function"]
    #[inline(always)] #[allow(non_snake_case)] fn
    init(mut c : init :: Context) -> (Shared, Local, init :: Monotonics)
    {
        let mut rcc =
        c.device.RCC.configure().hsi48().enable_crs(c.device.CRS).sysclk(48.mhz()).pclk(24.mhz()).freeze(&
        mut c.device.FLASH); let gpioa = c.device.GPIOA.split(& mut rcc); let
        gpiob = c.device.GPIOB.split(& mut rcc); let usb = usb :: Peripheral
        { usb : c.device.USB, pin_dm : gpioa.pa11, pin_dp : gpioa.pa12, }; *
        c.local.bus = Some(usb :: UsbBusType :: new(usb)); let usb_bus =
        c.local.bus.as_ref().unwrap(); let usb_class = keyberon ::
        new_class(usb_bus, ()); let usb_dev = keyberon :: new_device(usb_bus);
        let mut timer = timers :: Timer ::
        tim3(c.device.TIM3, 1.khz(), & mut rcc);
        timer.listen(timers :: Event :: TimeOut); let pb12 : & gpiob :: PB12 <
        Input < Floating > > = & gpiob.pb12; let is_left =
        pb12.is_low().get(); let transform : fn(Event) -> Event = if is_left
        { | e | e } else { | e | e.transform(| i, j | (i, 11 - j)) }; let
        (pa9, pa10) = (gpioa.pa9, gpioa.pa10); let pins = cortex_m ::
        interrupt ::
        free(move | cs |
        { (pa9.into_alternate_af1(cs), pa10.into_alternate_af1(cs)) }); let
        mut serial = serial :: Serial ::
        usart1(c.device.USART1, pins, 38_400.bps(), & mut rcc);
        serial.listen(serial :: Event :: Rxne); let (tx, rx) = serial.split();
        let pa15 = gpioa.pa15; let matrix = cortex_m :: interrupt ::
        free(move | cs |
        {
            Matrix ::
            new([pa15.into_pull_up_input(cs).downgrade(),
            gpiob.pb3.into_pull_up_input(cs).downgrade(),
            gpiob.pb4.into_pull_up_input(cs).downgrade(),
            gpiob.pb5.into_pull_up_input(cs).downgrade(),
            gpiob.pb8.into_pull_up_input(cs).downgrade(),
            gpiob.pb9.into_pull_up_input(cs).downgrade(),],
            [gpiob.pb0.into_push_pull_output(cs).downgrade(),
            gpiob.pb1.into_push_pull_output(cs).downgrade(),
            gpiob.pb2.into_push_pull_output(cs).downgrade(),
            gpiob.pb10.into_push_pull_output(cs).downgrade(),],)
        });
        (Shared
        {
            usb_dev, usb_class, layout : Layout ::
            new(& crate :: layout :: LAYERS),
        }, Local
        {
            timer, debouncer : Debouncer ::
            new([[false; 8]; 5], [[false; 8]; 5], 1), matrix : matrix.get(),
            transform, tx, rx, buf : [0; 5],
        }, init :: Monotonics(),)
    } #[doc = " User HW task: rx"] #[allow(non_snake_case)] fn
    rx(c : rx :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; if let
        Ok(b) = c.local.rx.read()
        {
            c.local.buf.rotate_left(1); c.local.buf [3] = b; if c.local.buf
            [3] == b'\n'
            {
                if let Ok(event) = de(& c.local.buf [..])
                { handle_event :: spawn(event).unwrap(); }
            }
        }
    } #[doc = " User HW task: usb_rx"] #[allow(non_snake_case)] fn
    usb_rx(c : usb_rx :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ;
        (c.shared.usb_dev,
        c.shared.usb_class).lock(| usb_dev, usb_class |
        { if usb_dev.poll(& mut [usb_class]) { usb_class.poll(); } });
    } #[doc = " User HW task: tick"] #[allow(non_snake_case)] fn
    tick(c : tick :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ;
        c.local.timer.wait().ok(); for event in
        c.local.debouncer.events(c.local.matrix.get().get()).map(c.local.transform)
        {
            for & b in & ser(event) { block! (c.local.tx.write(b)).get(); }
            handle_event :: spawn(event).unwrap();
        } tick_keyberon :: spawn().unwrap();
    } #[doc = " User SW task handle_event"] #[allow(non_snake_case)] fn
    handle_event(c : handle_event :: Context, event : Event)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ;
        c.shared.layout.event(event)
    } #[doc = " User SW task tick_keyberon"] #[allow(non_snake_case)] fn
    tick_keyberon(mut c : tick_keyberon :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; let tick =
        c.shared.layout.tick(); if c.shared.usb_dev.lock(| d | d.state()) !=
        UsbDeviceState :: Configured { return; } match tick
        {
            CustomEvent :: Release(()) => unsafe
            { cortex_m :: asm :: bootload(0x1FFFC800 as _) }, _ => (),
        } let report : KbHidReport = c.shared.layout.keycodes().collect(); if
        !
        c.shared.usb_class.lock(| k |
        k.device_mut().set_keyboard_report(report.clone())) { return; } while
        let Ok(0) = c.shared.usb_class.lock(| k | k.write(report.as_bytes()))
        {}
    } #[doc = " RTIC shared resource struct"] struct Shared
    {
        usb_dev : UsbDevice, usb_class : UsbClass, layout : Layout < 16, 5, 1,
        () > ,
    } #[doc = " RTIC local resource struct"] struct Local
    {
        matrix : Matrix < Pin < Input < PullUp > > , Pin < Output < PushPull >
        > , 8, 5 > , debouncer : Debouncer < [[bool; 8]; 5] > , timer : timers
        :: Timer < stm32 :: TIM3 > , transform : fn(Event) -> Event, tx :
        serial :: Tx < hal :: pac :: USART1 > , rx : serial :: Rx < hal :: pac
        :: USART1 > , buf : [u8; 5],
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Local resources `init` has access to"] pub struct
    __rtic_internal_initLocalResources < >
    {
        #[doc = " Local resource `bus`"] pub bus : & 'static mut Option <
        UsbBusAllocator < usb :: UsbBusType > > ,
    } #[doc = r" Monotonics used by the system"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_Monotonics();
    #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_init_Context <
    'a >
    {
        #[doc = r" Core (Cortex-M) peripherals"] pub core : rtic :: export ::
        Peripherals, #[doc = r" Device peripherals"] pub device : crate :: hal
        :: pac :: Peripherals, #[doc = r" Critical section token for init"]
        pub cs : rtic :: export :: CriticalSection < 'a > ,
        #[doc = r" Local Resources this task has access to"] pub local : init
        :: LocalResources < > ,
    } impl < 'a > __rtic_internal_init_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(core : rtic :: export :: Peripherals,) -> Self
        {
            __rtic_internal_init_Context
            {
                device : crate :: hal :: pac :: Peripherals :: steal(), cs :
                rtic :: export :: CriticalSection :: new(), core, local : init
                :: LocalResources :: new(),
            }
        }
    } #[allow(non_snake_case)] #[doc = " Initialization function"] pub mod
    init
    {
        #[doc(inline)] pub use super :: __rtic_internal_initLocalResources as
        LocalResources; #[doc(inline)] pub use super ::
        __rtic_internal_Monotonics as Monotonics; #[doc(inline)] pub use super
        :: __rtic_internal_init_Context as Context;
    } mod shared_resources
    {
        use rtic :: export :: Priority; #[doc(hidden)]
        #[allow(non_camel_case_types)] pub struct
        usb_dev_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > usb_dev_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { usb_dev_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        usb_class_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > usb_class_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { usb_class_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        }
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Local resources `rx` has access to"] pub struct
    __rtic_internal_rxLocalResources < 'a >
    {
        #[doc = " Local resource `rx`"] pub rx : & 'a mut serial :: Rx < hal
        :: pac :: USART1 > , #[doc = " Local resource `buf`"] pub buf : & 'a
        mut [u8; 5],
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_rx_Context < 'a
    >
    {
        #[doc = r" Local Resources this task has access to"] pub local : rx ::
        LocalResources < 'a > ,
    } impl < 'a > __rtic_internal_rx_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_rx_Context
            { local : rx :: LocalResources :: new(), }
        }
    } #[allow(non_snake_case)] #[doc = " Hardware task"] pub mod rx
    {
        #[doc(inline)] pub use super :: __rtic_internal_rxLocalResources as
        LocalResources; #[doc(inline)] pub use super ::
        __rtic_internal_rx_Context as Context;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Shared resources `usb_rx` has access to"] pub struct
    __rtic_internal_usb_rxSharedResources < 'a >
    {
        #[doc =
        " Resource proxy resource `usb_dev`. Use method `.lock()` to gain access"]
        pub usb_dev : shared_resources :: usb_dev_that_needs_to_be_locked < 'a
        > ,
        #[doc =
        " Resource proxy resource `usb_class`. Use method `.lock()` to gain access"]
        pub usb_class : shared_resources :: usb_class_that_needs_to_be_locked
        < 'a > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_usb_rx_Context <
    'a >
    {
        #[doc = r" Shared Resources this task has access to"] pub shared :
        usb_rx :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_usb_rx_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_usb_rx_Context
            { shared : usb_rx :: SharedResources :: new(priority), }
        }
    } #[allow(non_snake_case)] #[doc = " Hardware task"] pub mod usb_rx
    {
        #[doc(inline)] pub use super :: __rtic_internal_usb_rxSharedResources
        as SharedResources; #[doc(inline)] pub use super ::
        __rtic_internal_usb_rx_Context as Context;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Local resources `tick` has access to"] pub struct
    __rtic_internal_tickLocalResources < 'a >
    {
        #[doc = " Local resource `matrix`"] pub matrix : & 'a mut Matrix < Pin
        < Input < PullUp > > , Pin < Output < PushPull > > , 8, 5 > ,
        #[doc = " Local resource `debouncer`"] pub debouncer : & 'a mut
        Debouncer < [[bool; 8]; 5] > , #[doc = " Local resource `timer`"] pub
        timer : & 'a mut timers :: Timer < stm32 :: TIM3 > ,
        #[doc = " Local resource `transform`"] pub transform : & 'a mut
        fn(Event) -> Event, #[doc = " Local resource `tx`"] pub tx : & 'a mut
        serial :: Tx < hal :: pac :: USART1 > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_tick_Context <
    'a >
    {
        #[doc = r" Local Resources this task has access to"] pub local : tick
        :: LocalResources < 'a > ,
    } impl < 'a > __rtic_internal_tick_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_tick_Context
            { local : tick :: LocalResources :: new(), }
        }
    } #[allow(non_snake_case)] #[doc = " Hardware task"] pub mod tick
    {
        #[doc(inline)] pub use super :: __rtic_internal_tickLocalResources as
        LocalResources; #[doc(inline)] pub use super ::
        __rtic_internal_tick_Context as Context;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Shared resources `handle_event` has access to"] pub struct
    __rtic_internal_handle_eventSharedResources < 'a >
    {
        #[doc = " Lock free resource `layout`"] pub layout : & 'a mut Layout <
        16, 5, 1, () > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct
    __rtic_internal_handle_event_Context < 'a >
    {
        #[doc = r" Shared Resources this task has access to"] pub shared :
        handle_event :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_handle_event_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_handle_event_Context
            { shared : handle_event :: SharedResources :: new(priority), }
        }
    } #[doc = r" Spawns the task directly"] pub fn
    __rtic_internal_handle_event_spawn(_0 : Event,) -> Result < (), Event >
    {
        let input = _0; unsafe
        {
            if let Some(index) = rtic :: export :: interrupt ::
            free(| _ |
            (& mut * __rtic_internal_handle_event_FQ.get_mut()).dequeue())
            {
                (& mut *
                __rtic_internal_handle_event_INPUTS.get_mut()).get_unchecked_mut(usize
                :: from(index)).as_mut_ptr().write(input); rtic :: export ::
                interrupt ::
                free(| _ |
                {
                    (& mut *
                    __rtic_internal_P2_RQ.get_mut()).enqueue_unchecked((P2_T ::
                    handle_event, index));
                }); rtic :: pend(crate :: hal :: pac :: interrupt :: CEC_CAN);
                Ok(())
            } else { Err(input) }
        }
    } #[allow(non_snake_case)] #[doc = " Software task"] pub mod handle_event
    {
        #[doc(inline)] pub use super ::
        __rtic_internal_handle_eventSharedResources as SharedResources;
        #[doc(inline)] pub use super :: __rtic_internal_handle_event_Context
        as Context; #[doc(inline)] pub use super ::
        __rtic_internal_handle_event_spawn as spawn;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Shared resources `tick_keyberon` has access to"] pub struct
    __rtic_internal_tick_keyberonSharedResources < 'a >
    {
        #[doc =
        " Resource proxy resource `usb_dev`. Use method `.lock()` to gain access"]
        pub usb_dev : shared_resources :: usb_dev_that_needs_to_be_locked < 'a
        > ,
        #[doc =
        " Resource proxy resource `usb_class`. Use method `.lock()` to gain access"]
        pub usb_class : shared_resources :: usb_class_that_needs_to_be_locked
        < 'a > , #[doc = " Lock free resource `layout`"] pub layout : & 'a mut
        Layout < 16, 5, 1, () > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct
    __rtic_internal_tick_keyberon_Context < 'a >
    {
        #[doc = r" Shared Resources this task has access to"] pub shared :
        tick_keyberon :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_tick_keyberon_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_tick_keyberon_Context
            { shared : tick_keyberon :: SharedResources :: new(priority), }
        }
    } #[doc = r" Spawns the task directly"] pub fn
    __rtic_internal_tick_keyberon_spawn() -> Result < (), () >
    {
        let input = (); unsafe
        {
            if let Some(index) = rtic :: export :: interrupt ::
            free(| _ |
            (& mut * __rtic_internal_tick_keyberon_FQ.get_mut()).dequeue())
            {
                (& mut *
                __rtic_internal_tick_keyberon_INPUTS.get_mut()).get_unchecked_mut(usize
                :: from(index)).as_mut_ptr().write(input); rtic :: export ::
                interrupt ::
                free(| _ |
                {
                    (& mut *
                    __rtic_internal_P2_RQ.get_mut()).enqueue_unchecked((P2_T ::
                    tick_keyberon, index));
                }); rtic :: pend(crate :: hal :: pac :: interrupt :: CEC_CAN);
                Ok(())
            } else { Err(input) }
        }
    } #[allow(non_snake_case)] #[doc = " Software task"] pub mod tick_keyberon
    {
        #[doc(inline)] pub use super ::
        __rtic_internal_tick_keyberonSharedResources as SharedResources;
        #[doc(inline)] pub use super :: __rtic_internal_tick_keyberon_Context
        as Context; #[doc(inline)] pub use super ::
        __rtic_internal_tick_keyberon_spawn as spawn;
    } #[doc = r" App module"] impl < > __rtic_internal_initLocalResources < >
    {
        #[inline(always)] #[doc(hidden)] pub unsafe fn new() -> Self
        {
            __rtic_internal_initLocalResources
            { bus : & mut * __rtic_internal_local_init_bus.get_mut(), }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic0"] static
    __rtic_internal_shared_resource_usb_dev : rtic :: RacyCell < core :: mem
    :: MaybeUninit < UsbDevice >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: usb_dev_that_needs_to_be_locked < 'a >
    {
        type T = UsbDevice; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut UsbDevice) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 3u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_usb_dev.get_mut() as *
                mut _, self.priority(), CEILING, crate :: hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic1"] static
    __rtic_internal_shared_resource_usb_class : rtic :: RacyCell < core :: mem
    :: MaybeUninit < UsbClass >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: usb_class_that_needs_to_be_locked < 'a >
    {
        type T = UsbClass; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut UsbClass) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 3u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_usb_class.get_mut() as *
                mut _, self.priority(), CEILING, crate :: hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic2"] static
    __rtic_internal_shared_resource_layout : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Layout < 16, 5, 1, () > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); #[doc(hidden)]
    #[allow(non_upper_case_globals)] const __rtic_internal_MASK_CHUNKS : usize
    = rtic :: export ::
    compute_mask_chunks([crate :: hal :: pac :: Interrupt :: CEC_CAN as u32,
    crate :: hal :: pac :: Interrupt :: USART1 as u32, crate :: hal :: pac ::
    Interrupt :: USB as u32, crate :: hal :: pac :: Interrupt :: TIM3 as
    u32]); #[doc(hidden)] #[allow(non_upper_case_globals)] const
    __rtic_internal_MASKS :
    [rtic :: export :: Mask < __rtic_internal_MASK_CHUNKS > ; 3] =
    [rtic :: export ::
    create_mask([crate :: hal :: pac :: Interrupt :: TIM3 as u32]), rtic ::
    export ::
    create_mask([crate :: hal :: pac :: Interrupt :: CEC_CAN as u32]), rtic ::
    export :: create_mask([crate :: hal :: pac :: Interrupt :: USB as u32])];
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic3"] static
    __rtic_internal_local_resource_matrix : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Matrix < Pin < Input < PullUp > > , Pin < Output < PushPull
    > > , 8, 5 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic4"] static
    __rtic_internal_local_resource_debouncer : rtic :: RacyCell < core :: mem
    :: MaybeUninit < Debouncer < [[bool; 8]; 5] > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic5"] static
    __rtic_internal_local_resource_timer : rtic :: RacyCell < core :: mem ::
    MaybeUninit < timers :: Timer < stm32 :: TIM3 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic6"] static
    __rtic_internal_local_resource_transform : rtic :: RacyCell < core :: mem
    :: MaybeUninit < fn(Event) -> Event >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic7"] static
    __rtic_internal_local_resource_tx : rtic :: RacyCell < core :: mem ::
    MaybeUninit < serial :: Tx < hal :: pac :: USART1 > >> = rtic :: RacyCell
    :: new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic8"] static
    __rtic_internal_local_resource_rx : rtic :: RacyCell < core :: mem ::
    MaybeUninit < serial :: Rx < hal :: pac :: USART1 > >> = rtic :: RacyCell
    :: new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic9"] static
    __rtic_internal_local_resource_buf : rtic :: RacyCell < core :: mem ::
    MaybeUninit < [u8; 5] >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] static __rtic_internal_local_init_bus : rtic :: RacyCell <
    Option < UsbBusAllocator < usb :: UsbBusType > > > = rtic :: RacyCell ::
    new(None); #[allow(non_snake_case)] #[no_mangle]
    #[doc = " User HW task ISR trampoline for rx"] unsafe fn USART1()
    {
        const PRIORITY : u8 = 4u8; rtic :: export ::
        run(PRIORITY, ||
        {
            rx(rx :: Context ::
            new(& rtic :: export :: Priority :: new(PRIORITY)))
        });
    } impl < 'a > __rtic_internal_rxLocalResources < 'a >
    {
        #[inline(always)] #[doc(hidden)] pub unsafe fn new() -> Self
        {
            __rtic_internal_rxLocalResources
            {
                rx : & mut *
                (& mut *
                __rtic_internal_local_resource_rx.get_mut()).as_mut_ptr(), buf
                : & mut *
                (& mut *
                __rtic_internal_local_resource_buf.get_mut()).as_mut_ptr(),
            }
        }
    } #[allow(non_snake_case)] #[no_mangle]
    #[doc = " User HW task ISR trampoline for usb_rx"] unsafe fn USB()
    {
        const PRIORITY : u8 = 3u8; rtic :: export ::
        run(PRIORITY, ||
        {
            usb_rx(usb_rx :: Context ::
            new(& rtic :: export :: Priority :: new(PRIORITY)))
        });
    } impl < 'a > __rtic_internal_usb_rxSharedResources < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_usb_rxSharedResources
            {
                #[doc(hidden)] usb_dev : shared_resources ::
                usb_dev_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] usb_class : shared_resources ::
                usb_class_that_needs_to_be_locked :: new(priority),
            }
        }
    } #[allow(non_snake_case)] #[no_mangle]
    #[doc = " User HW task ISR trampoline for tick"] unsafe fn TIM3()
    {
        const PRIORITY : u8 = 1u8; rtic :: export ::
        run(PRIORITY, ||
        {
            tick(tick :: Context ::
            new(& rtic :: export :: Priority :: new(PRIORITY)))
        });
    } impl < 'a > __rtic_internal_tickLocalResources < 'a >
    {
        #[inline(always)] #[doc(hidden)] pub unsafe fn new() -> Self
        {
            __rtic_internal_tickLocalResources
            {
                matrix : & mut *
                (& mut *
                __rtic_internal_local_resource_matrix.get_mut()).as_mut_ptr(),
                debouncer : & mut *
                (& mut *
                __rtic_internal_local_resource_debouncer.get_mut()).as_mut_ptr(),
                timer : & mut *
                (& mut *
                __rtic_internal_local_resource_timer.get_mut()).as_mut_ptr(),
                transform : & mut *
                (& mut *
                __rtic_internal_local_resource_transform.get_mut()).as_mut_ptr(),
                tx : & mut *
                (& mut *
                __rtic_internal_local_resource_tx.get_mut()).as_mut_ptr(),
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] static __rtic_internal_handle_event_FQ : rtic :: RacyCell <
    rtic :: export :: SCFQ < 9 > > = rtic :: RacyCell ::
    new(rtic :: export :: Queue :: new()); #[link_section = ".uninit.rtic10"]
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] static __rtic_internal_handle_event_INPUTS : rtic ::
    RacyCell < [core :: mem :: MaybeUninit < Event > ; 8] > = rtic :: RacyCell
    ::
    new([core :: mem :: MaybeUninit :: uninit(), core :: mem :: MaybeUninit ::
    uninit(), core :: mem :: MaybeUninit :: uninit(), core :: mem ::
    MaybeUninit :: uninit(), core :: mem :: MaybeUninit :: uninit(), core ::
    mem :: MaybeUninit :: uninit(), core :: mem :: MaybeUninit :: uninit(),
    core :: mem :: MaybeUninit :: uninit(),]); impl < 'a >
    __rtic_internal_handle_eventSharedResources < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_handle_eventSharedResources
            {
                #[doc = " Exclusive access resource `layout`"] layout : & mut
                *
                (& mut *
                __rtic_internal_shared_resource_layout.get_mut()).as_mut_ptr(),
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] static __rtic_internal_tick_keyberon_FQ : rtic :: RacyCell
    < rtic :: export :: SCFQ < 2 > > = rtic :: RacyCell ::
    new(rtic :: export :: Queue :: new()); #[link_section = ".uninit.rtic11"]
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] static __rtic_internal_tick_keyberon_INPUTS : rtic ::
    RacyCell < [core :: mem :: MaybeUninit < () > ; 1] > = rtic :: RacyCell ::
    new([core :: mem :: MaybeUninit :: uninit(),]); impl < 'a >
    __rtic_internal_tick_keyberonSharedResources < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_tick_keyberonSharedResources
            {
                #[doc(hidden)] usb_dev : shared_resources ::
                usb_dev_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] usb_class : shared_resources ::
                usb_class_that_needs_to_be_locked :: new(priority),
                #[doc = " Exclusive access resource `layout`"] layout : & mut
                *
                (& mut *
                __rtic_internal_shared_resource_layout.get_mut()).as_mut_ptr(),
            }
        }
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[derive(Clone, Copy)] #[doc(hidden)] pub enum P2_T
    { handle_event, tick_keyberon, } #[doc(hidden)]
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)] static
    __rtic_internal_P2_RQ : rtic :: RacyCell < rtic :: export :: SCRQ < P2_T,
    10 > > = rtic :: RacyCell :: new(rtic :: export :: Queue :: new());
    #[allow(non_snake_case)]
    #[doc = "Interrupt handler to dispatch tasks at priority 2"] #[no_mangle]
    unsafe fn CEC_CAN()
    {
        #[doc = r" The priority of this interrupt handler"] const PRIORITY :
        u8 = 2u8; rtic :: export ::
        run(PRIORITY, ||
        {
            while let Some((task, index)) =
            (& mut * __rtic_internal_P2_RQ.get_mut()).split().1.dequeue()
            {
                match task
                {
                    P2_T :: handle_event =>
                    {
                        let _0 =
                        (& *
                        __rtic_internal_handle_event_INPUTS.get()).get_unchecked(usize
                        :: from(index)).as_ptr().read();
                        (& mut *
                        __rtic_internal_handle_event_FQ.get_mut()).split().0.enqueue_unchecked(index);
                        let priority = & rtic :: export :: Priority ::
                        new(PRIORITY);
                        handle_event(handle_event :: Context :: new(priority), _0)
                    } P2_T :: tick_keyberon =>
                    {
                        let () =
                        (& *
                        __rtic_internal_tick_keyberon_INPUTS.get()).get_unchecked(usize
                        :: from(index)).as_ptr().read();
                        (& mut *
                        __rtic_internal_tick_keyberon_FQ.get_mut()).split().0.enqueue_unchecked(index);
                        let priority = & rtic :: export :: Priority ::
                        new(PRIORITY);
                        tick_keyberon(tick_keyberon :: Context :: new(priority))
                    }
                }
            }
        });
    } #[doc(hidden)] mod rtic_ext
    {
        use super :: * ; #[no_mangle] unsafe extern "C" fn main() -> !
        {
            rtic :: export :: assert_send :: < UsbDevice > (); rtic :: export
            :: assert_send :: < UsbClass > (); rtic :: export :: assert_send
            :: < Layout < 16, 5, 1, () > > (); rtic :: export :: assert_send
            :: < Matrix < Pin < Input < PullUp > > , Pin < Output < PushPull >
            > , 8, 5 > > (); rtic :: export :: assert_send :: < Debouncer <
            [[bool; 8]; 5] > > (); rtic :: export :: assert_send :: < timers
            :: Timer < stm32 :: TIM3 > > (); rtic :: export :: assert_send ::
            < fn(Event) -> Event > (); rtic :: export :: assert_send :: <
            serial :: Tx < hal :: pac :: USART1 > > (); rtic :: export ::
            assert_send :: < serial :: Rx < hal :: pac :: USART1 > > (); rtic
            :: export :: assert_send :: < [u8; 5] > (); rtic :: export ::
            assert_send :: < Event > (); const _CONST_CHECK : () =
            {
                if ! rtic :: export :: have_basepri()
                {
                    if (crate :: hal :: pac :: Interrupt :: USART1 as usize) >=
                    (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        :: core :: panic!
                        ("An interrupt out of range is used while in armv6 or armv8m.base");
                    } if (crate :: hal :: pac :: Interrupt :: USB as usize) >=
                    (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        :: core :: panic!
                        ("An interrupt out of range is used while in armv6 or armv8m.base");
                    } if (crate :: hal :: pac :: Interrupt :: TIM3 as usize) >=
                    (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        :: core :: panic!
                        ("An interrupt out of range is used while in armv6 or armv8m.base");
                    }
                } else {}
            }; let _ = _CONST_CHECK; rtic :: export :: interrupt :: disable();
            (0 ..
            8u8).for_each(| i |
            (& mut *
            __rtic_internal_handle_event_FQ.get_mut()).enqueue_unchecked(i));
            (0 ..
            1u8).for_each(| i |
            (& mut *
            __rtic_internal_tick_keyberon_FQ.get_mut()).enqueue_unchecked(i));
            let mut core : rtic :: export :: Peripherals = rtic :: export ::
            Peripherals :: steal().into(); let _ =
            you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml ::
            interrupt :: CEC_CAN; const _ : () = if
            (1 << crate :: hal :: pac :: NVIC_PRIO_BITS) < 2u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'CEC_CAN' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: CEC_CAN, rtic :: export ::
            logical2hw(2u8, crate :: hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: CEC_CAN); const _ : () = if
            (1 << crate :: hal :: pac :: NVIC_PRIO_BITS) < 4u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'USART1' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: USART1, rtic :: export ::
            logical2hw(4u8, crate :: hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: USART1); const _ : () = if
            (1 << crate :: hal :: pac :: NVIC_PRIO_BITS) < 3u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'USB' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: USB, rtic :: export ::
            logical2hw(3u8, crate :: hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: USB); const _ : () = if
            (1 << crate :: hal :: pac :: NVIC_PRIO_BITS) < 1u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'TIM3' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: TIM3, rtic :: export ::
            logical2hw(1u8, crate :: hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: TIM3); #[inline(never)] fn __rtic_init_resources <
            F > (f : F) where F : FnOnce() { f(); }
            __rtic_init_resources(||
            {
                let (shared_resources, local_resources, mut monotonics) =
                init(init :: Context :: new(core.into()));
                __rtic_internal_shared_resource_usb_dev.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.usb_dev));
                __rtic_internal_shared_resource_usb_class.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.usb_class));
                __rtic_internal_shared_resource_layout.get_mut().write(core ::
                mem :: MaybeUninit :: new(shared_resources.layout));
                __rtic_internal_local_resource_matrix.get_mut().write(core ::
                mem :: MaybeUninit :: new(local_resources.matrix));
                __rtic_internal_local_resource_debouncer.get_mut().write(core
                :: mem :: MaybeUninit :: new(local_resources.debouncer));
                __rtic_internal_local_resource_timer.get_mut().write(core ::
                mem :: MaybeUninit :: new(local_resources.timer));
                __rtic_internal_local_resource_transform.get_mut().write(core
                :: mem :: MaybeUninit :: new(local_resources.transform));
                __rtic_internal_local_resource_tx.get_mut().write(core :: mem
                :: MaybeUninit :: new(local_resources.tx));
                __rtic_internal_local_resource_rx.get_mut().write(core :: mem
                :: MaybeUninit :: new(local_resources.rx));
                __rtic_internal_local_resource_buf.get_mut().write(core :: mem
                :: MaybeUninit :: new(local_resources.buf)); rtic :: export ::
                interrupt :: enable();
            }); loop { rtic :: export :: nop() }
        }
    }
}