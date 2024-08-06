
#[derive(Clone, Debug, Default)]
pub struct VisibilityZone {
    pub start: usize, // line number where the symbol is defined
    pub end: usize, // line number where the symbol can no longer be seen (start of a new function or end of the file)
}

impl VisibilityZone {
    pub fn is_visible_at(&self, line_number: usize) -> bool {
        line_number >= self.start && (self.end == 0 || line_number < self.end)
    }
}
