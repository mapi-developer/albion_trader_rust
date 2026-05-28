use std::collections::HashMap;

pub struct FragmentBuffer {
    // Map of sequence_number -> (Total Expected Fragments, Fragments Received, Assembled Data Buffer)
    buffers: HashMap<u32, (u32, u32, Vec<u8>)>,
}

impl FragmentBuffer {
    pub fn new() -> Self {
        FragmentBuffer {
            buffers: HashMap::new(),
        }
    }

    /// Adds a fragment to the buffer. Returns the fully assembled payload if complete, or None if still waiting.
    pub fn offer(
        &mut self,
        sequence_number: u32,
        fragment_count: u32,
        total_length: u32,
        fragment_offset: u32,
        payload: &[u8],
    ) -> Option<Vec<u8>> {
        
        // Find the buffer for this sequence number, or create a new one if it's the first piece
        let entry = self.buffers.entry(sequence_number).or_insert_with(|| {
            // Pre-allocate the exact size needed for the final combined packet
            (fragment_count, 0, vec![0; total_length as usize])
        });

        // Copy this fragment's data into the correct offset of our master buffer
        let start = fragment_offset as usize;
        let end = start + payload.len();
        entry.2[start..end].copy_from_slice(payload);
        
        // Increment the count of received fragments
        entry.1 += 1;

        // If we have received all fragments, remove it from the map and return the assembled data
        if entry.1 == entry.0 {
            let complete_data = self.buffers.remove(&sequence_number).unwrap().2;
            return Some(complete_data);
        }

        None
    }
}