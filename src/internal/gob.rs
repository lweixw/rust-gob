use bytes::{BigEndian, Buf, BufMut};

#[derive(Debug)]
pub(crate) enum Error {
    IncompleteMessage,
    IntegerOverflow
}

pub(crate) struct Message<B> {
    buf: B
}

impl<B> Message<B> {
    pub fn new(buf: B) -> Message<B> {
        Message { buf }
    }

    pub fn get_ref(&self) -> &B {
        &self.buf
    }

    pub fn get_mut(&mut self) -> &mut B {
        &mut self.buf
    }
}

impl<B: Buf> Message<B> {
    #[inline]
    pub fn read_uint(&mut self) -> Result<u64, Error> {
        if self.buf.remaining() < 1 {
            return Err(Error::IncompleteMessage);
        }
        let u7_or_len = self.buf.get_u8();
        if u7_or_len < 128 {
            return Ok(u7_or_len as u64);
        }
        let len = !u7_or_len + 1;
        if self.buf.remaining() < len as usize {
            return Err(Error::IncompleteMessage);
        }
        Ok(self.buf.get_uint::<BigEndian>(len as usize))
    }

    #[inline]
    pub fn read_int(&mut self) -> Result<i64, Error> {
        let bits = self.read_uint()?;
        let sign = bits & 1;
        let sint = (bits >> 1) as i64;
        if sign == 0 {
            Ok(sint)
        } else {
            Ok(!sint)
        }
    }

    #[inline]
    pub fn read_float(&mut self) -> Result<f64, Error> {
        let bits = self.read_uint()?;
        Ok(f64::from_bits(bits.swap_bytes()))
    }

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool, Error> {
        match self.read_uint()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::IntegerOverflow)
        }
    }

    #[inline]
    pub fn read_bytes_len(&mut self) -> Result<usize, Error> {
        let len = self.read_uint()?;
        if (self.buf.remaining() as u64) < len {
            return Err(Error::IncompleteMessage);
        }
        Ok(len as usize)
    }
}

impl<B: BufMut> Message<B> {
    #[inline]
    pub fn write_uint(&mut self, n: u64) -> Result<(), Error> {
        if n < 128 {
            self.buf.put_u8(n as u8);
        } else {
            let nbytes = 8 - (n.leading_zeros() / 8) as u8;
            self.buf.put_u8(!(nbytes - 1));
            self.buf.put_uint::<BigEndian>(n, nbytes as usize);
        }
        Ok(())
    }

    #[inline]
    pub fn write_bool(&mut self, b: bool) -> Result<(), Error> {
        match b {
            false => self.write_uint(0),
            true => self.write_uint(1)
        }
    }

    #[inline]
    pub fn write_int(&mut self, n: i64) -> Result<(), Error> {
        let u: u64;
        if n < 0 {
		    u = (!(n as u64) << 1) | 1;
        } else {
            u = (n as u64) << 1;
        }
        self.write_uint(u)
    }

    #[inline]
    pub fn write_float(&mut self, n: f64) -> Result<(), Error> {
        let bits = n.to_bits();
        self.write_uint(bits.swap_bytes())
    }

    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.write_uint(bytes.len() as u64)?;
        self.buf.put_slice(bytes);
        Ok(())
    }
}