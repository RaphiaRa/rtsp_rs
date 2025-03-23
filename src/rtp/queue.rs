use super::Packet;
use std::collections::BinaryHeap;

pub struct ReorderQueue {
    queue: BinaryHeap<Packet>,
    max_len: usize,
    last_read_sn: u16,
}

impl ReorderQueue {
    pub fn new(max_len: usize) -> Self {
        Self {
            queue: BinaryHeap::new(),
            max_len,
            last_read_sn: 0,
        }
    }

    pub fn pop(&mut self) -> Option<Packet> {
        if let Some(packet) = self.queue.peek() {
            if packet.sequence_number() == self.last_read_sn + 1 || self.queue.len() >= self.max_len {
                self.last_read_sn = packet.sequence_number();
                return self.queue.pop();
            }
        }
        None
    }

    /// pushes a packet to the queue if it is not too old
    /// or returns the packet again if it is the next in line
    pub fn push_or_return(&mut self, packet: Packet) -> Option<Packet> {
        if self.last_read_sn == 0 || packet.sequence_number() == self.last_read_sn + 1 {
            self.last_read_sn = packet.sequence_number();
            Some(packet)
        } else if packet.sequence_number() < self.last_read_sn {
            log::warn!("Packet too old, discarding");
            None
        } else {
            self.queue.push(packet);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_reorder_queue() {
        // Create buffer with 5 rtp packets (with 2 byte size prefix)
        // ts and ssrc are set to 0
        // sequence number is set to 23, 25, 27, 24, 26
        let mut packet_bufs = vec![
            vec![0x80, 0x60, 0x00, 0x17, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // seq 23
            vec![0x80, 0x60, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // seq 25
            vec![0x80, 0x60, 0x00, 0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // seq 27
            vec![0x80, 0x60, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // seq 24
            vec![0x80, 0x60, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // seq 26
        ];
        let mut reorder_queue = ReorderQueue::new(5);
        assert_eq!(
            reorder_queue
                .push_or_return(Packet::new(packet_bufs.remove(0)).unwrap())
                .unwrap()
                .sequence_number(),
            23
        );
        assert!(reorder_queue
            .push_or_return(Packet::new(packet_bufs.remove(0)).unwrap())
            .is_none());
        assert!(reorder_queue
            .push_or_return(Packet::new(packet_bufs.remove(0)).unwrap())
            .is_none());
        assert_eq!(
            reorder_queue
                .push_or_return(Packet::new(packet_bufs.remove(0)).unwrap())
                .unwrap()
                .sequence_number(),
            24
        );
        assert!(reorder_queue
            .push_or_return(Packet::new(packet_bufs.remove(0)).unwrap())
            .is_none());
        assert_eq!(reorder_queue.pop().unwrap().sequence_number(), 25);
        assert_eq!(reorder_queue.pop().unwrap().sequence_number(), 26);
        assert_eq!(reorder_queue.pop().unwrap().sequence_number(), 27);
        assert!(reorder_queue.pop().is_none());
    }
}
