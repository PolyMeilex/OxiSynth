use crate::voice::Voice;

pub struct VoicePool {
    voices: Vec<Voice>,
}

impl VoicePool {
    pub fn new(len: usize, output_rate: f32) -> Self {
        let mut voices = Vec::new();

        for _ in 0..len {
            voices.push(Voice::new(output_rate))
        }

        Self { voices }
    }

    pub fn len(&self) -> usize {
        self.voices.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Voice> {
        self.voices.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Voice> {
        self.voices.iter_mut()
    }

    pub fn clear(&mut self) {
        self.voices.clear();
    }
}

impl std::ops::Index<usize> for VoicePool {
    type Output = Voice;
    fn index(&self, id: usize) -> &Self::Output {
        &self.voices[id]
    }
}

impl std::ops::IndexMut<usize> for VoicePool {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        &mut self.voices[id]
    }
}
