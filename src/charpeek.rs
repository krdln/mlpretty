use ::std::io::prelude::*;

macro_rules! otry{
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => return None,
        }
    }
}

/// Yields lines from a reader, and allows to peek one byte
/// from the next line.
pub struct Charpeek<R: BufRead> {
    reader: R,
    peek: Option<[u8;1]>
}

impl<R: BufRead> Charpeek<R> {
    pub fn new(r: R) -> Self {
        Charpeek { reader: r, peek: None }
    }

    pub fn peek_byte(&mut self) -> Option<u8> {
        match self.peek {
            Some(x) => Some(x[0]),
            None => {
                let mut buf = [0u8];
                otry!( self.reader.read_exact(&mut buf).ok() );
                self.peek = Some(buf);
                Some(buf[0])
            }
        }
    }

    pub fn flush_peek<W: Write>(&mut self, mut writer: W) {
        {
            let peek = self.peek.as_ref().map_or(&[] as &[u8], |x| x);
            let _ = writer.write(&peek);
            let _ = writer.flush();
        }
        self.peek = None;
    }

    pub fn next_line(&mut self) -> Option<String> {
        let mut line = String::new();
        {
            let peek = self.peek.as_ref().map_or(&[] as &[u8], |x| x);
            otry!( peek.chain(&mut self.reader).read_line(&mut line).ok() );
        }
        if line.len() == 0 { return None; }
        self.peek = None;
        let tru_len = line.trim_right().len();
        line.truncate(tru_len);
        Some(line)
    }
}
