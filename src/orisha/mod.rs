use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrishaVector {
    #[serde(rename = "Esu")]
    pub esu: f64,
    #[serde(rename = "Ogun")]
    pub ogun: f64,
    #[serde(rename = "Oya")]
    pub oya: f64,
    #[serde(rename = "Obatala")]
    pub obatala: f64,
    #[serde(rename = "Oshun")]
    pub oshun: f64,
    #[serde(rename = "Shango")]
    pub shango: f64,
    #[serde(rename = "Yemoja")]
    pub yemoja: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orisha {
    Esu,
    Ogun,
    Oya,
    Obatala,
    Oshun,
    Shango,
    Yemoja,
}

impl Default for OrishaVector {
    fn default() -> Self {
        OrishaVector {
            esu: 0.14,
            ogun: 0.14,
            oya: 0.14,
            obatala: 0.14,
            oshun: 0.14,
            shango: 0.14,
            yemoja: 0.14,
        }
    }
}

impl OrishaVector {
    pub fn from_odu_day(odu_id: u16, day: &crate::cosmogram::Day) -> OrishaVector {
        // Derive from SHA-256 hash of odu_id + day string
        let day_str = format!("{:?}", day);
        let input = format!("{}:{}", odu_id, day_str);
        let hash = Sha256::digest(input.as_bytes());

        let to_f64 = |b: u8| (b as f64) / 255.0;

        OrishaVector {
            esu: to_f64(hash[0]),
            ogun: to_f64(hash[1]),
            oya: to_f64(hash[2]),
            obatala: to_f64(hash[3]),
            oshun: to_f64(hash[4]),
            shango: to_f64(hash[5]),
            yemoja: to_f64(hash[6]),
        }
    }

    pub fn from_archetype(name: &str) -> OrishaVector {
        match name.to_lowercase().as_str() {
            "esu" | "eshu" | "elegba" | "legba" => OrishaVector {
                esu: 0.9, ogun: 0.1, oya: 0.1, obatala: 0.1, oshun: 0.1, shango: 0.1, yemoja: 0.1,
            },
            "ogun" | "ogún" => OrishaVector {
                esu: 0.1, ogun: 0.9, oya: 0.2, obatala: 0.1, oshun: 0.1, shango: 0.3, yemoja: 0.1,
            },
            "oya" | "ọya" => OrishaVector {
                esu: 0.2, ogun: 0.2, oya: 0.9, obatala: 0.1, oshun: 0.2, shango: 0.4, yemoja: 0.2,
            },
            "obatala" | "ọbàtálá" => OrishaVector {
                esu: 0.1, ogun: 0.1, oya: 0.1, obatala: 0.9, oshun: 0.2, shango: 0.1, yemoja: 0.2,
            },
            "oshun" | "ọshun" | "osun" | "ọ̀ṣun" => OrishaVector {
                esu: 0.2, ogun: 0.1, oya: 0.2, obatala: 0.2, oshun: 0.9, shango: 0.1, yemoja: 0.3,
            },
            "shango" | "sango" | "ṣàngó" => OrishaVector {
                esu: 0.2, ogun: 0.3, oya: 0.4, obatala: 0.1, oshun: 0.1, shango: 0.9, yemoja: 0.1,
            },
            "yemoja" | "yemanja" => OrishaVector {
                esu: 0.1, ogun: 0.1, oya: 0.2, obatala: 0.2, oshun: 0.3, shango: 0.1, yemoja: 0.9,
            },
            _ => OrishaVector::default(),
        }
    }

    pub fn dominant(&self) -> Option<Orisha> {
        let values = [
            (self.esu, Orisha::Esu),
            (self.ogun, Orisha::Ogun),
            (self.oya, Orisha::Oya),
            (self.obatala, Orisha::Obatala),
            (self.oshun, Orisha::Oshun),
            (self.shango, Orisha::Shango),
            (self.yemoja, Orisha::Yemoja),
        ];

        values
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, o)| o)
    }

    pub fn scale(&mut self, factor: f64) {
        self.esu *= factor;
        self.ogun *= factor;
        self.oya *= factor;
        self.obatala *= factor;
        self.oshun *= factor;
        self.shango *= factor;
        self.yemoja *= factor;
    }

    pub fn normalize(&mut self) {
        let sum = self.esu + self.ogun + self.oya + self.obatala + self.oshun + self.shango + self.yemoja;
        if sum > 0.0 {
            self.scale(1.0 / sum);
        }
    }
}
