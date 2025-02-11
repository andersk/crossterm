use std::borrow::ToOwned;
use std::io::{self, Error, Result};
use std::str;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::NULL;
use winapi::um::consoleapi::{GetNumberOfConsoleInputEvents, ReadConsoleInputW, WriteConsoleW};
use winapi::um::{
    wincon::{
        FillConsoleOutputAttribute, FillConsoleOutputCharacterA, GetLargestConsoleWindowSize,
        SetConsoleTextAttribute, SetConsoleWindowInfo, COORD, INPUT_RECORD, SMALL_RECT,
    },
    winnt::HANDLE,
};

use super::{is_true, Coord, Handle, HandleType, InputRecord, WindowPositions};

/// Could be used to do some basic things with the console.
pub struct Console {
    handle: Handle,
}

impl Console {
    /// Create new instance of `Console`.
    ///
    /// This created instance will use the default output handle (STD_OUTPUT_HANDLE) as handle for the function call it wraps.
    pub fn new() -> Result<Console> {
        Ok(Console {
            handle: Handle::new(HandleType::OutputHandle)?,
        })
    }

    /// Sets the attributes of characters written to the console screen buffer by the WriteFile or WriteConsole function, or echoed by the ReadFile or ReadConsole function.
    /// This function affects text written after the function call.
    ///
    /// parameter: [wAttributes]
    /// Wraps the underlying function call: [SetConsoleTextAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletextattribute]
    pub fn set_text_attribute(&self, value: u16) -> Result<()> {
        unsafe {
            if !is_true(SetConsoleTextAttribute(*self.handle, value)) {
                return Err(Error::last_os_error());
            }
        }
        Ok(())
    }

    /// Sets the current size and position of a console screen buffer's window.
    ///
    /// Wraps the underlying function call: [SetConsoleTextAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/setconsoletextattribute]
    pub fn set_console_info(&self, absolute: bool, rect: WindowPositions) -> Result<()> {
        let absolute = match absolute {
            true => 1,
            false => 0,
        };
        let a = SMALL_RECT::from(rect);

        unsafe {
            if !is_true(SetConsoleWindowInfo(*self.handle, absolute, &a)) {
                return Err(Error::last_os_error());
            }
        }

        Ok(())
    }

    /// Writes a character to the console screen buffer a specified number of times, beginning at the specified coordinates
    ///
    /// Wraps the underlying function call: [FillConsoleOutputCharacterA]
    /// link: [https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputcharacter]
    pub fn fill_whit_character(
        &self,
        start_location: Coord,
        cells_to_write: u32,
        filling_char: char,
    ) -> Result<u32> {
        let mut chars_written = 0;
        unsafe {
            // fill the cells in console with blanks
            if !is_true(FillConsoleOutputCharacterA(
                *self.handle,
                filling_char as i8,
                cells_to_write,
                COORD::from(start_location),
                &mut chars_written,
            )) {
                return Err(Error::last_os_error());
            }

            Ok(chars_written)
        }
    }

    /// Sets the character attributes for a specified number of character cells, beginning at the specified coordinates in a screen buffer.
    ///
    /// Wraps the underlying function call: [FillConsoleOutputAttribute]
    /// link: [https://docs.microsoft.com/en-us/windows/console/fillconsoleoutputattribute]
    pub fn fill_whit_attribute(
        &self,
        start_location: Coord,
        cells_to_write: u32,
        dw_attribute: u16,
    ) -> Result<u32> {
        let mut cells_written = 0;
        // Get the position of the current console window
        unsafe {
            if !is_true(FillConsoleOutputAttribute(
                *self.handle,
                dw_attribute,
                cells_to_write,
                COORD::from(start_location),
                &mut cells_written,
            )) {
                return Err(Error::last_os_error());
            }
        }

        Ok(cells_written)
    }

    /// Retrieves the size of the largest possible console window, based on the current text and the size of the display.
    ///
    /// Wraps the underlying function call: [GetLargestConsoleWindowSize]
    /// link: [https://docs.microsoft.com/en-us/windows/console/getlargestconsolewindowsize]
    pub fn largest_window_size(&self) -> Coord {
        Coord::from(unsafe { GetLargestConsoleWindowSize(*self.handle) })
    }

    /// Writes a character string to a console screen buffer beginning at the current cursor location.
    ///
    /// Wraps the underlying function call: [WriteConsoleW]
    /// link: [https://docs.microsoft.com/en-us/windows/console/writeconsole]
    pub fn write_char_buffer(&self, buf: &[u8]) -> Result<usize> {
        // get string from u8[] and parse it to an c_str
        let utf8 = match str::from_utf8(buf) {
            Ok(string) => string,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not parse to utf8 string",
                ));
            }
        };

        let utf16: Vec<u16> = utf8.encode_utf16().collect();
        let utf16_ptr: *const c_void = utf16.as_ptr() as *const _ as *const c_void;

        let mut cells_written: u32 = 0;
        // write to console
        unsafe {
            if !is_true(WriteConsoleW(
                *self.handle,
                utf16_ptr,
                utf16.len() as u32,
                &mut cells_written,
                NULL,
            )) {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(utf8.as_bytes().len())
    }

    pub fn read_single_input_event(&self) -> Result<Option<InputRecord>> {
        let buf_len = self.number_of_console_input_events()?;

        // Fast-skipping all the code below if there is nothing to read at all
        if buf_len == 0 {
            return Ok(None);
        }

        let mut buf: Vec<INPUT_RECORD> = Vec::with_capacity(1);
        let mut size = 0;

        let a = self.read_input(&mut buf, 1, &mut size)?.1[0].to_owned();

        // read single input event
        Ok(Some(a))
    }

    pub fn read_console_input(&self) -> Result<(u32, Vec<InputRecord>)> {
        let buf_len = self.number_of_console_input_events()?;

        // Fast-skipping all the code below if there is nothing to read at all
        if buf_len == 0 {
            return Ok((0, vec![]));
        }

        let mut buf: Vec<INPUT_RECORD> = Vec::with_capacity(buf_len as usize);
        let mut size = 0;

        self.read_input(&mut buf, buf_len, &mut size)
    }

    pub fn number_of_console_input_events(&self) -> Result<u32> {
        let mut buf_len: DWORD = 0;
        if !is_true(unsafe { GetNumberOfConsoleInputEvents(*self.handle, &mut buf_len) }) {
            return Err(Error::last_os_error());
        }

        Ok(buf_len)
    }

    fn read_input(
        &self,
        buf: &mut Vec<INPUT_RECORD>,
        buf_len: u32,
        bytes_written: &mut u32,
    ) -> Result<(u32, Vec<InputRecord>)> {
        if !is_true(unsafe {
            ReadConsoleInputW(*self.handle, buf.as_mut_ptr(), buf_len, bytes_written)
        }) {
            return Err(Error::last_os_error());
        } else {
            unsafe {
                buf.set_len(buf_len as usize);
            }
        }

        Ok((
            buf_len,
            buf[..(buf_len as usize)]
                .iter()
                .map(|x| InputRecord::from(*x))
                .collect::<Vec<InputRecord>>(),
        ))
    }
}

impl From<Handle> for Console {
    /// Create a `Console` instance who's functions will be executed on the the given `Handle`
    fn from(handle: Handle) -> Self {
        Console { handle }
    }
}

impl From<HANDLE> for Console {
    /// Create a `Console` instance who's functions will be executed on the the given `HANDLE`
    fn from(handle: HANDLE) -> Self {
        Console {
            handle: Handle::from(handle),
        }
    }
}
