use crate::*;

pub struct HCenter<E: Element>(pub E);

impl<E: Element> Element for HCenter<E> {
    fn insufficient_first_height(&self, ctx: InsufficientFirstHeightCtx) -> bool {
        self.0
            .insufficient_first_height(InsufficientFirstHeightCtx {
                width: WidthConstraint {
                    expand: false,
                    ..ctx.width
                },
                ..ctx
            })
    }

    fn measure(&self, ctx: MeasureCtx) -> Option<ElementSize> {
        let width = ctx.width;

        let size = self.0.measure(MeasureCtx {
            width: WidthConstraint {
                expand: false,
                max: width.max,
            },
            ..ctx
        });

        size.map(|size| ElementSize {
            width: width.constrain(size.width),
            height: size.height,
        })
    }

    fn draw(&self, mut ctx: DrawCtx) -> Option<ElementSize> {
        let width = ctx.width;

        let size = if width.expand {
            let mut break_count = 0;
            let mut extra_location_min_height = 0.;

            let element_size = self.0.measure(MeasureCtx {
                width: WidthConstraint {
                    max: width.max,
                    expand: false,
                },
                first_height: ctx.first_height,
                breakable: ctx.breakable.as_ref().map(|b| BreakableMeasure {
                    full_height: b.full_height,
                    break_count: &mut break_count,
                    extra_location_min_height: &mut extra_location_min_height,
                }),
            });

            let x_offset;
            let element_width;

            if let Some(size) = element_size {
                x_offset = (width.max - size.width) / 2.0;
                element_width = size.width;
            } else {
                x_offset = 0.;
                element_width = ctx.width.max;
            }

            ctx.location.pos.0 += x_offset;

            let width_constraint = WidthConstraint {
                max: element_width,
                expand: true,
            };

            if let Some(breakable) = ctx.breakable {
                self.0.draw(DrawCtx {
                    width: width_constraint,
                    breakable: Some(BreakableDraw {
                        full_height: breakable.full_height,
                        preferred_height_break_count: breakable.preferred_height_break_count,
                        get_location: &mut |pdf, location_id| {
                            let mut location = (breakable.get_location)(pdf, location_id);

                            location.pos.0 += x_offset;

                            location
                        },
                    }),
                    ..ctx
                })
            } else {
                self.0.draw(DrawCtx {
                    width: width_constraint,
                    breakable: None,
                    ..ctx
                })
            }
        } else {
            self.0.draw(ctx)
        };

        size.map(|size| ElementSize {
            width: width.constrain(size.width),
            height: size.height,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        elements::none::NoneElement,
        test_utils::{BuildElement, ElementProxy, ElementTestParams, FakeImage, FakeText},
    };

    use super::*;

    #[test]
    fn test_h_center_none() {
        for output in ElementTestParams::default().run(&HCenter(NoneElement)) {
            output.assert_size(None);

            if let Some(b) = output.breakable {
                b.assert_break_count(0).assert_extra_location_min_height(0.);
            }
        }
    }

    #[test]
    fn test_h_center_fake_text() {
        let element = BuildElement(|build_ctx, callback| {
            let content = FakeText {
                width: 5.,
                line_height: 1.,
                lines: 10,
            };

            let proxy = ElementProxy {
                before_draw: &|ctx: &mut DrawCtx| {
                    assert_eq!(
                        ctx.width,
                        WidthConstraint {
                            max: if build_ctx.width.expand { 5. } else { 6. },
                            expand: build_ctx.width.expand,
                        }
                    );
                    assert_eq!(
                        ctx.location.pos.0,
                        12. + if ctx.width.expand { 0.5 } else { 0. }
                    );
                },
                after_break: &|_location_idx: u32,
                               location: &Location,
                               width: WidthConstraint,
                               _first_height| {
                    assert_eq!(location.pos.0, 12. + if width.expand { 0.5 } else { 0. });
                },
                ..ElementProxy::new(content)
            };
            callback.call(HCenter(proxy))
        });

        for output in (ElementTestParams {
            first_height: 1.,
            full_height: 2.,
            width: 6.,
            ..Default::default()
        })
        .run(&element)
        {
            output.assert_size(Some(ElementSize {
                width: output.width.constrain(5.),
                height: Some(if output.breakable.is_none() {
                    10.
                } else if output.first_height == 1. {
                    1.
                } else {
                    2.
                }),
            }));

            if let Some(b) = output.breakable {
                b.assert_break_count(if output.first_height == 1. { 5 } else { 4 })
                    .assert_extra_location_min_height(0.);
            }
        }
    }

    #[test]
    fn test_h_center_too_wide() {
        // If the element wants to be wider than the width constraint the element should just get
        // the width constraint.

        let element = BuildElement(|build_ctx, callback| {
            let content = FakeText {
                width: 5.,
                line_height: 1.,
                lines: 2,
            };

            let proxy = ElementProxy {
                before_draw: &|ctx: &mut DrawCtx| {
                    assert_eq!(
                        ctx.width,
                        WidthConstraint {
                            max: 4.,
                            expand: build_ctx.width.expand,
                        }
                    );
                    assert_eq!(ctx.location.pos.0, 12.);
                },
                after_break: &|_location_idx: u32,
                               location: &Location,
                               _width: WidthConstraint,
                               _first_height| {
                    assert_eq!(location.pos.0, 12.);
                },
                ..ElementProxy::new(content)
            };
            callback.call(HCenter(proxy))
        });

        for output in (ElementTestParams {
            first_height: 1.,
            full_height: 2.,
            width: 4.,
            ..Default::default()
        })
        .run(&element)
        {
            output.assert_size(Some(ElementSize {
                width: output.width.constrain(5.),
                height: Some(if output.breakable.is_none() {
                    2.
                } else if output.first_height == 1. {
                    1.
                } else {
                    2.
                }),
            }));

            if let Some(b) = output.breakable {
                b.assert_break_count(if output.first_height == 1. { 1 } else { 0 })
                    .assert_extra_location_min_height(0.);
            }
        }
    }

    #[test]
    fn test_overdraw() {
        struct Overdraw;

        impl Element for Overdraw {
            fn measure(&self, _: MeasureCtx) -> Option<ElementSize> {
                Some(ElementSize {
                    width: 5.,
                    height: None,
                })
            }

            fn draw(&self, _: DrawCtx) -> Option<ElementSize> {
                Some(ElementSize {
                    width: 5.,
                    height: None,
                })
            }
        }

        for output in (ElementTestParams {
            first_height: 1.,
            full_height: 2.,
            width: 4.,
            ..Default::default()
        })
        .run(&HCenter(Overdraw))
        {
            output.assert_size(Some(ElementSize {
                width: 4.,
                height: None,
            }));

            if let Some(b) = output.breakable {
                b.assert_break_count(0).assert_extra_location_min_height(0.);
            }
        }
    }

    #[test]
    fn test_h_center_fake_image() {
        let element = BuildElement(|build_ctx, callback| {
            let content = FakeImage {
                width: 5.,
                height: 2.,
            };

            let proxy = ElementProxy {
                before_draw: &|ctx: &mut DrawCtx| {
                    assert_eq!(
                        ctx.width,
                        WidthConstraint {
                            max: if build_ctx.width.expand { 5. } else { 10. },
                            expand: build_ctx.width.expand,
                        }
                    );
                    assert_eq!(
                        ctx.location.pos.0,
                        12. + if ctx.width.expand { 2.5 } else { 0. }
                    );
                },
                after_break: &|_location_idx: u32,
                               location: &Location,
                               width: WidthConstraint,
                               _first_height| {
                    assert_eq!(location.pos.0, 12. + if width.expand { 2.5 } else { 0. });
                },
                ..ElementProxy::new(content)
            };
            callback.call(HCenter(proxy))
        });

        for output in (ElementTestParams {
            first_height: 1.,
            full_height: 4.,
            width: 10.,
            ..Default::default()
        })
        .run(&element)
        {
            output.assert_size(Some(ElementSize {
                width: output.width.constrain(5.),
                height: Some(2.),
            }));

            if let Some(b) = output.breakable {
                b.assert_break_count(if output.first_height == 1. { 1 } else { 0 })
                    .assert_extra_location_min_height(0.);
            }
        }
    }

    #[test]
    fn test_h_center_fake_image_too_wide() {
        let element = BuildElement(|build_ctx, callback| {
            let content = FakeImage {
                width: 10.,
                height: 4.,
            };

            let proxy = ElementProxy {
                before_draw: &|ctx: &mut DrawCtx| {
                    assert_eq!(
                        ctx.width,
                        WidthConstraint {
                            max: 5.,
                            expand: build_ctx.width.expand,
                        }
                    );
                    assert_eq!(ctx.location.pos.0, 12.);
                },
                after_break: &|_location_idx: u32,
                               location: &Location,
                               _width: WidthConstraint,
                               _first_height| {
                    assert_eq!(location.pos.0, 12.);
                },
                ..ElementProxy::new(content)
            };
            callback.call(HCenter(proxy))
        });

        for output in (ElementTestParams {
            first_height: 1.,
            full_height: 4.,
            width: 5.,
            ..Default::default()
        })
        .run(&element)
        {
            output.assert_size(Some(ElementSize {
                width: 5.,
                height: Some(2.),
            }));

            if let Some(b) = output.breakable {
                b.assert_break_count(if output.first_height == 1. { 1 } else { 0 })
                    .assert_extra_location_min_height(0.);
            }
        }
    }
}
