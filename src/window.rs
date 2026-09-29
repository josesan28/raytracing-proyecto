#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::c_void;
    use std::io;
    use std::mem::size_of;
    use std::ptr::{null, null_mut};

    use crate::camera::Camera;
    use crate::color::Color;
    use crate::framebuffer::Framebuffer;
    use crate::math::Vec3;
    use crate::output::write_bmp;
    use crate::renderer::render;
    use crate::scene::Scene;

    type Handle = *mut c_void;
    type WindowHandle = Handle;
    type DeviceContext = Handle;
    type InstanceHandle = Handle;

    const WINDOW_WIDTH: i32 = 960;
    const WINDOW_HEIGHT: i32 = 580;
    const RENDER_WIDTH: usize = 800;
    const RENDER_HEIGHT: usize = 450;

    const WM_DESTROY: u32 = 0x0002;
    const WM_SIZE: u32 = 0x0005;
    const WM_PAINT: u32 = 0x000F;
    const WM_CLOSE: u32 = 0x0010;
    const WM_QUIT: u32 = 0x0012;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_MOUSEWHEEL: u32 = 0x020A;

    const VK_ESCAPE: usize = 0x1B;
    const VK_LEFT: usize = 0x25;
    const VK_UP: usize = 0x26;
    const VK_RIGHT: usize = 0x27;
    const VK_DOWN: usize = 0x28;
    const VK_ADD: usize = 0x6B;
    const VK_SUBTRACT: usize = 0x6D;
    const VK_OEM_PLUS: usize = 0xBB;
    const VK_OEM_MINUS: usize = 0xBD;

    const CS_OWNDC: u32 = 0x0020;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
    const WS_VISIBLE: u32 = 0x10000000;
    const CW_USEDEFAULT: i32 = i32::MIN;
    const PM_REMOVE: u32 = 0x0001;
    const SW_SHOW: i32 = 5;
    const DIB_RGB_COLORS: u32 = 0;
    const SRCCOPY: u32 = 0x00CC0020;
    const BI_RGB: u32 = 0;
    const IDC_ARROW: usize = 32512;

    #[repr(C)]
    struct WindowClass {
        style: u32,
        window_procedure:
            Option<unsafe extern "system" fn(WindowHandle, u32, usize, isize) -> isize>,
        class_extra: i32,
        window_extra: i32,
        instance: InstanceHandle,
        icon: Handle,
        cursor: Handle,
        background: Handle,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Message {
        window: WindowHandle,
        message: u32,
        word_parameter: usize,
        long_parameter: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    impl Default for Message {
        fn default() -> Self {
            Self {
                window: null_mut(),
                message: 0,
                word_parameter: 0,
                long_parameter: 0,
                time: 0,
                point: Point::default(),
                private: 0,
            }
        }
    }

    #[repr(C)]
    #[derive(Default)]
    struct Rectangle {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct PaintData {
        context: DeviceContext,
        erase_background: i32,
        paint_rectangle: Rectangle,
        restore: i32,
        incremental_update: i32,
        reserved: [u8; 32],
    }

    impl Default for PaintData {
        fn default() -> Self {
            Self {
                context: null_mut(),
                erase_background: 0,
                paint_rectangle: Rectangle::default(),
                restore: 0,
                incremental_update: 0,
                reserved: [0; 32],
            }
        }
    }

    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        image_size: u32,
        x_pixels_per_meter: i32,
        y_pixels_per_meter: i32,
        colors_used: u32,
        important_colors: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn RegisterClassW(window_class: *const WindowClass) -> u16;
        fn CreateWindowExW(
            extended_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: WindowHandle,
            menu: Handle,
            instance: InstanceHandle,
            parameter: *mut c_void,
        ) -> WindowHandle;
        fn DefWindowProcW(
            window: WindowHandle,
            message: u32,
            word_parameter: usize,
            long_parameter: isize,
        ) -> isize;
        fn DestroyWindow(window: WindowHandle) -> i32;
        fn PostQuitMessage(exit_code: i32);
        fn PeekMessageW(
            message: *mut Message,
            window: WindowHandle,
            minimum: u32,
            maximum: u32,
            remove: u32,
        ) -> i32;
        fn TranslateMessage(message: *const Message) -> i32;
        fn DispatchMessageW(message: *const Message) -> isize;
        fn ShowWindow(window: WindowHandle, command: i32) -> i32;
        fn UpdateWindow(window: WindowHandle) -> i32;
        fn GetClientRect(window: WindowHandle, rectangle: *mut Rectangle) -> i32;
        fn ValidateRect(window: WindowHandle, rectangle: *const Rectangle) -> i32;
        fn BeginPaint(window: WindowHandle, paint_data: *mut PaintData) -> DeviceContext;
        fn EndPaint(window: WindowHandle, paint_data: *const PaintData) -> i32;
        fn GetDC(window: WindowHandle) -> DeviceContext;
        fn ReleaseDC(window: WindowHandle, context: DeviceContext) -> i32;
        fn LoadCursorW(instance: InstanceHandle, cursor_name: *const u16) -> Handle;
        fn SetCapture(window: WindowHandle) -> WindowHandle;
        fn ReleaseCapture() -> i32;
        fn WaitMessage() -> i32;
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn StretchDIBits(
            context: DeviceContext,
            destination_x: i32,
            destination_y: i32,
            destination_width: i32,
            destination_height: i32,
            source_x: i32,
            source_y: i32,
            source_width: i32,
            source_height: i32,
            bits: *const c_void,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            raster_operation: u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> InstanceHandle;
    }

    unsafe extern "system" fn window_procedure(
        window: WindowHandle,
        message: u32,
        word_parameter: usize,
        long_parameter: isize,
    ) -> isize {
        match message {
            WM_PAINT => {
                let mut paint_data = PaintData::default();
                unsafe {
                    BeginPaint(window, &mut paint_data);
                    EndPaint(window, &paint_data);
                }
                0
            }
            WM_CLOSE => {
                unsafe { DestroyWindow(window) };
                0
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(window, message, word_parameter, long_parameter) },
        }
    }

    pub fn run(scene: &Scene) -> io::Result<()> {
        let class_name = wide_string("CpuRaytracerWindow");
        let title = wide_string(
            "Ruinas del templo | Arrastra: rotar | Rueda: zoom | WASD: rotar | P: guardar",
        );
        let instance = unsafe { GetModuleHandleW(null()) };
        let window_class = WindowClass {
            style: CS_OWNDC,
            window_procedure: Some(window_procedure),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: null_mut(),
            cursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW as *const u16) },
            background: null_mut(),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };

        if unsafe { RegisterClassW(&window_class) } == 0 {
            return Err(io::Error::last_os_error());
        }

        let window = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            )
        };

        if window.is_null() {
            return Err(io::Error::last_os_error());
        }

        unsafe {
            ShowWindow(window, SW_SHOW);
            UpdateWindow(window);
        }

        let mut camera = initial_camera();
        let background = Color::new(8, 10, 18);
        let mut framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, background);
        let mut display_pixels = vec![0_u32; RENDER_WIDTH * RENDER_HEIGHT];
        let mut needs_render = true;
        let mut needs_present = true;
        let mut running = true;
        let mut dragging = false;
        let mut previous_mouse = (0_i32, 0_i32);

        while running {
            let mut message = Message::default();
            while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if message.message == WM_QUIT {
                    running = false;
                    break;
                }

                match message.message {
                    WM_SIZE | WM_PAINT => needs_present = true,
                    WM_KEYDOWN => {
                        if handle_key(message.word_parameter, &mut camera, scene)? {
                            needs_render = true;
                        }
                        if message.word_parameter == VK_ESCAPE {
                            unsafe { DestroyWindow(window) };
                        }
                    }
                    WM_LBUTTONDOWN => {
                        dragging = true;
                        previous_mouse = mouse_position(message.long_parameter);
                        unsafe { SetCapture(window) };
                    }
                    WM_LBUTTONUP => {
                        dragging = false;
                        unsafe { ReleaseCapture() };
                    }
                    WM_MOUSEMOVE if dragging => {
                        let current_mouse = mouse_position(message.long_parameter);
                        let delta_x = current_mouse.0 - previous_mouse.0;
                        let delta_y = current_mouse.1 - previous_mouse.1;
                        if delta_x != 0 || delta_y != 0 {
                            camera.orbit(-(delta_x as f32) * 0.006, -(delta_y as f32) * 0.006);
                            previous_mouse = current_mouse;
                            needs_render = true;
                        }
                    }
                    WM_MOUSEWHEEL => {
                        let wheel_delta = ((message.word_parameter >> 16) as u16 as i16) as f32;
                        camera.zoom(wheel_delta / 120.0 * 0.7);
                        needs_render = true;
                    }
                    _ => {}
                }

                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            if !running {
                break;
            }

            if needs_render {
                render(&mut framebuffer, &camera, scene);
                convert_pixels(&framebuffer, &mut display_pixels);
                needs_render = false;
                needs_present = true;
            }

            if needs_present {
                draw_frame(window, &display_pixels, RENDER_WIDTH, RENDER_HEIGHT)?;
                needs_present = false;
            }

            if !needs_render && !needs_present {
                unsafe { WaitMessage() };
            }
        }

        Ok(())
    }

    fn initial_camera() -> Camera {
        use std::f32::consts::PI;

        Camera::new(
            Vec3::new(3.65, 2.55, 2.15),
            Vec3::new(0.0, 0.75, -5.7),
            Vec3::new(0.0, 1.0, 0.0),
            PI / 3.5,
        )
    }

    fn handle_key(key: usize, camera: &mut Camera, scene: &Scene) -> io::Result<bool> {
        match key {
            VK_LEFT | 0x41 => camera.orbit(-0.12, 0.0),
            VK_RIGHT | 0x44 => camera.orbit(0.12, 0.0),
            VK_UP | 0x57 => camera.orbit(0.0, 0.10),
            VK_DOWN | 0x53 => camera.orbit(0.0, -0.10),
            VK_ADD | VK_OEM_PLUS => camera.zoom(0.7),
            VK_SUBTRACT | VK_OEM_MINUS => camera.zoom(-0.7),
            0x52 => *camera = initial_camera(),
            0x50 => {
                save_view(scene, camera)?;
                return Ok(false);
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn save_view(scene: &Scene, camera: &Camera) -> io::Result<()> {
        let mut framebuffer = Framebuffer::new(800, 450, Color::new(8, 10, 18));
        render(&mut framebuffer, camera, scene);
        write_bmp("window_view.bmp", &framebuffer)
    }

    fn convert_pixels(framebuffer: &Framebuffer, output: &mut [u32]) {
        for (destination, source) in output.iter_mut().zip(framebuffer.pixels()) {
            *destination =
                (u32::from(source.r) << 16) | (u32::from(source.g) << 8) | u32::from(source.b);
        }
    }

    fn draw_frame(
        window: WindowHandle,
        pixels: &[u32],
        source_width: usize,
        source_height: usize,
    ) -> io::Result<()> {
        let mut rectangle = Rectangle::default();
        if unsafe { GetClientRect(window, &mut rectangle) } == 0 {
            return Err(io::Error::last_os_error());
        }

        let context = unsafe { GetDC(window) };
        if context.is_null() {
            return Err(io::Error::last_os_error());
        }

        let bitmap_info = BitmapInfo {
            header: BitmapInfoHeader {
                size: size_of::<BitmapInfoHeader>() as u32,
                width: source_width as i32,
                height: -(source_height as i32),
                planes: 1,
                bit_count: 32,
                compression: BI_RGB,
                image_size: (source_width * source_height * 4) as u32,
                x_pixels_per_meter: 0,
                y_pixels_per_meter: 0,
                colors_used: 0,
                important_colors: 0,
            },
            colors: [0],
        };

        unsafe {
            StretchDIBits(
                context,
                0,
                0,
                rectangle.right - rectangle.left,
                rectangle.bottom - rectangle.top,
                0,
                0,
                source_width as i32,
                source_height as i32,
                pixels.as_ptr().cast(),
                &bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
            ReleaseDC(window, context);
            ValidateRect(window, null());
        }

        Ok(())
    }

    fn mouse_position(long_parameter: isize) -> (i32, i32) {
        let x = (long_parameter as u16 as i16) as i32;
        let y = ((long_parameter >> 16) as u16 as i16) as i32;
        (x, y)
    }

    fn wide_string(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "windows")]
pub use platform::run;

#[cfg(not(target_os = "windows"))]
pub fn run(_scene: &crate::scene::Scene) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "El modo de ventana requiere Windows",
    ))
}
