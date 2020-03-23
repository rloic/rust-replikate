use std::ops::Deref;

pub trait TSVSerializable {
    fn to_tsv_format(&self) -> String;
}

macro_rules! tsv_serializable {
    ($x: ty) => {
        impl TSVSerializable for $x {
            fn to_tsv_format(&self) -> String {
                self.to_string()
            }
        }
    }
}

tsv_serializable!(bool);
tsv_serializable!(u8);
tsv_serializable!(i8);
tsv_serializable!(u16);
tsv_serializable!(i16);
tsv_serializable!(u32);
tsv_serializable!(i32);
tsv_serializable!(u64);
tsv_serializable!(i64);
tsv_serializable!(u128);
tsv_serializable!(i128);
tsv_serializable!(usize);
tsv_serializable!(f32);
tsv_serializable!(f64);
tsv_serializable!(char);

impl TSVSerializable for String {
    fn to_tsv_format(&self) -> String {
        format!("\"{}\"", self.replace("\"", "\"\"")).to_owned()
    }
}

impl TSVSerializable for &str {
    fn to_tsv_format(&self) -> String {
        format!("\"{}\"", self.replace("\"", "\"\"")).to_owned()
    }
}

impl <T> TSVSerializable for Vec<T> where T: TSVSerializable {
    fn to_tsv_format(&self) -> String {
        let mut result = String::new();

        if self.len() != 0 {
            result.push_str(&self[0].to_tsv_format());
            for i in 1 .. self.len() {
                result.push('\t');
                result.push_str(&self[i].to_tsv_format());
            }
        }

        result
    }
}

impl TSVSerializable for Box<dyn TSVSerializable> {
    fn to_tsv_format(&self) -> String {
        self.deref().to_tsv_format()
    }
}

impl <T> TSVSerializable for Box<T> where T: TSVSerializable {
    fn to_tsv_format(&self) -> String {
        self.deref().to_tsv_format()
    }
}