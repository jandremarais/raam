use crate::line;

pub(crate) struct Grid {
    pub(crate) nrows: usize,
    pub(crate) ncols: usize,
    pub(crate) row_height: f32,
    pub(crate) col_width: f32,
    pub(crate) line_width: f32,
}

impl Grid {
    pub(crate) fn new(
        nrows: usize,
        ncols: usize,
        row_height: f32,
        col_width: f32,
        line_width: f32,
    ) -> Self {
        Self {
            nrows,
            ncols,
            row_height,
            col_width,
            line_width,
        }
    }

    pub(crate) fn nlines(&self) -> usize {
        self.nrows + self.ncols + 2
    }

    pub(crate) fn size(&self) -> (f32, f32) {
        (
            self.ncols as f32 * self.col_width + self.line_width,
            self.nrows as f32 * self.row_height + self.line_width,
        )
    }

    pub(crate) fn line_instances(&self) -> Vec<line::Instance> {
        let (xlim, ylim) = self.size();
        let hlines: Vec<_> = (0..self.nrows + 1)
            .map(|i| {
                line::Instance::new(
                    (0., i as f32 * self.row_height),
                    (xlim, self.line_width),
                    255.,
                )
            })
            .collect();
        let vlines: Vec<_> = (0..self.ncols + 1)
            .map(|i| {
                line::Instance::new(
                    (i as f32 * self.col_width, 0.),
                    (self.line_width, ylim),
                    255.,
                )
            })
            .collect();
        let mut line_instances = Vec::new();
        line_instances.extend_from_slice(&hlines);
        line_instances.extend_from_slice(&vlines);
        line_instances
    }
}
