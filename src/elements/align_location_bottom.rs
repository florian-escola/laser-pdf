use crate::*;

pub struct AlignLocationBottom<'a, E: Element>(pub &'a E);

impl<'a, E: Element> Element for AlignLocationBottom<'a, E> {
    fn first_location_usage(&self, ctx: FirstLocationUsageCtx) -> FirstLocationUsage {
        let layout = self.layout(ctx.width, ctx.first_height, Some(ctx.full_height), 0);

        if layout.breaks > 0 {
            FirstLocationUsage::WillSkip
        } else if layout.size.height.is_some() {
            FirstLocationUsage::WillUse
        } else {
            FirstLocationUsage::NoneHeight
        }
    }

    fn measure(&self, mut ctx: MeasureCtx) -> ElementSize {
        let layout = self.layout(
            ctx.width,
            ctx.first_height,
            ctx.breakable.as_ref().map(|b| b.full_height),
            0,
        );

        let height = if layout.breaks > 0 {
            let breakable = ctx.breakable.as_mut().unwrap();

            *breakable.break_count = layout.breaks;

            Some(breakable.full_height)
        } else {
            layout.size.height.map(|_| ctx.first_height)
        };

        if let Some(breakable) = ctx.breakable {
            *breakable.extra_location_min_height = Some(breakable.full_height);
        }

        ElementSize {
            width: layout.size.width,
            height,
        }
    }

    fn draw(&self, ctx: DrawCtx) -> ElementSize {
        let layout = self.layout(
            ctx.width,
            ctx.first_height,
            ctx.breakable.as_ref().map(|b| b.full_height),
            ctx.breakable
                .as_ref()
                .map(|b| b.preferred_height_break_count)
                .unwrap_or(0),
        );

        let height_available;
        let height;
        let mut location;

        if layout.breaks > 0 {
            let breakable = ctx.breakable.unwrap();

            location = (breakable.get_location)(ctx.pdf, layout.breaks - 1);
            height_available = breakable.full_height;

            height = Some(breakable.full_height);
        } else {
            location = ctx.location;
            height_available = ctx.first_height;
            height = layout.size.height.map(|_| ctx.first_height);
        }

        location.pos.1 -= layout.y_offset;

        self.0.draw(DrawCtx {
            pdf: ctx.pdf,
            location,
            width: ctx.width,
            first_height: height_available,
            preferred_height: None,
            breakable: None,
        });

        ElementSize {
            width: layout.size.width,
            height,
        }
    }
}

#[derive(Debug)]
struct Layout {
    breaks: u32,
    y_offset: f64,
    size: ElementSize,
}

impl<'a, E: Element> AlignLocationBottom<'a, E> {
    fn layout(
        &self,
        width: WidthConstraint,
        first_height: f64,
        full_height: Option<f64>,
        preferred_breaks: u32,
    ) -> Layout {
        let height_available = full_height.unwrap_or(first_height);

        let size = self.0.measure(MeasureCtx {
            width,
            first_height: height_available,
            breakable: None,
        });

        let breaks;
        let location_height;

        if let (Some(height), Some(full_height)) = (size.height, full_height) {
            breaks = if preferred_breaks == 0 && height > first_height {
                1
            } else {
                preferred_breaks
            };

            location_height = if breaks > 0 {
                full_height
            } else {
                first_height
            };
        } else {
            breaks = 0;
            location_height = first_height;
        };

        let y_offset = if let Some(height) = size.height {
            location_height - height
        } else {
            0.
        };

        Layout {
            breaks,
            y_offset,
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{record_passes::RecordPasses, *};
    use insta::*;

    #[test]
    fn test_unbreakable() {
        let width = WidthConstraint {
            max: 12.,
            expand: true,
        };
        let first_height = 21.;
        let pos = (11., 29.0);

        let element = BuildElement(|ctx, callback| {
            let content = RecordPasses::new(FakeText {
                lines: 3,
                line_height: 5.,
                width: 3.,
            });

            let element = AlignLocationBottom(&content);

            let ret = callback.call(element);

            if let build_element::Pass::Draw {
                preferred_height: None,
                ..
            } = ctx.pass
            {
                assert_debug_snapshot!(content.into_passes());
            }

            ret
        });

        let output = test_element(
            &element,
            TestElementParams {
                width,
                first_height,
                breakable: None,
                pos,
                ..Default::default()
            },
        );

        assert_debug_snapshot!(output);
    }

    #[test]
    fn test_breakable() {
        let width = WidthConstraint {
            max: 12.,
            expand: true,
        };
        let first_height = 21.;
        let full_height = 25.;
        let pos = (11., 29.0);

        let element = BuildElement(|ctx, callback| {
            let content = RecordPasses::new(FakeText {
                lines: 3,
                line_height: 5.,
                width: 3.,
            });

            let element = AlignLocationBottom(&content);

            let ret = callback.call(element);

            if let build_element::Pass::Draw {
                preferred_height: None,
                ..
            } = ctx.pass
            {
                assert_debug_snapshot!(content.into_passes());
            }

            ret
        });

        let output = test_element(
            &element,
            TestElementParams {
                width,
                first_height,
                breakable: Some(TestElementParamsBreakable {
                    full_height,
                    ..Default::default()
                }),
                pos,
                ..Default::default()
            },
        );

        assert_debug_snapshot!(output);
    }

    #[test]
    fn test_pre_break() {
        let width = WidthConstraint {
            max: 12.,
            expand: true,
        };
        let first_height = 21.;
        let full_height = 26.;
        let pos = (11., 29.0);

        let element = BuildElement(|ctx, callback| {
            let content = RecordPasses::new(FakeText {
                lines: 5,
                line_height: 5.,
                width: 3.,
            });

            let element = AlignLocationBottom(&content);

            let ret = callback.call(element);

            if let build_element::Pass::Draw {
                preferred_height: None,
                ..
            } = ctx.pass
            {
                assert_debug_snapshot!(content.into_passes());
            }

            ret
        });

        let output = test_element(
            &element,
            TestElementParams {
                width,
                first_height,
                breakable: Some(TestElementParamsBreakable {
                    full_height,
                    ..Default::default()
                }),
                pos,
                ..Default::default()
            },
        );

        assert_debug_snapshot!(output);
    }

    #[test]
    fn test_preferred_height() {
        let width = WidthConstraint {
            max: 12.,
            expand: true,
        };
        let first_height = 21.;
        let full_height = 26.;
        let pos = (11., 29.0);

        let element = BuildElement(|ctx, callback| {
            let content = RecordPasses::new(FakeText {
                lines: 5,
                line_height: 5.,
                width: 3.,
            });

            let element = AlignLocationBottom(&content);

            let ret = callback.call(element);

            if let build_element::Pass::Draw {
                preferred_height: None,
                ..
            } = ctx.pass
            {
                assert_debug_snapshot!(content.into_passes());
            }

            ret
        });

        let output = test_element(
            &element,
            TestElementParams {
                width,
                first_height,
                breakable: Some(TestElementParamsBreakable {
                    full_height,
                    preferred_height_break_count: 4,
                }),
                pos,
                preferred_height: None,
                ..Default::default()
            },
        );

        assert_debug_snapshot!(output);
    }
}
