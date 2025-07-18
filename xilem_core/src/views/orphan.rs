// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    DynMessage, MessageResult, Mut, View, ViewElement, ViewId, ViewMarker, ViewPathTracker,
};

/// This trait provides a way to add [`View`] implementations for types that would be restricted otherwise by the orphan rules.
///
/// Every type that can be supported with this trait, needs a concrete `View` implementation in `xilem_core`, possibly feature-gated.
pub trait OrphanView<V, State, Action>: ViewPathTracker + Sized {
    /// See [`View::Element`]
    type OrphanElement: ViewElement;
    /// See [`View::ViewState`]
    type OrphanViewState;

    /// See [`View::build`]
    fn orphan_build(
        view: &V,
        ctx: &mut Self,
        app_state: &mut State,
    ) -> (Self::OrphanElement, Self::OrphanViewState);

    /// See [`View::rebuild`]
    fn orphan_rebuild(
        new: &V,
        prev: &V,
        view_state: &mut Self::OrphanViewState,
        ctx: &mut Self,
        element: Mut<'_, Self::OrphanElement>,
        app_state: &mut State,
    );

    /// See [`View::teardown`]
    fn orphan_teardown(
        view: &V,
        view_state: &mut Self::OrphanViewState,
        ctx: &mut Self,
        element: Mut<'_, Self::OrphanElement>,
        app_state: &mut State,
    );

    /// See [`View::message`]
    fn orphan_message(
        view: &V,
        view_state: &mut Self::OrphanViewState,
        id_path: &[ViewId],
        message: DynMessage,
        app_state: &mut State,
    ) -> MessageResult<Action>;
}

macro_rules! impl_orphan_view_for {
    ($ty: ty) => {
        impl ViewMarker for $ty {}

        impl<State, Action, Context> View<State, Action, Context> for $ty
        where
            Context: OrphanView<$ty, State, Action>,
        {
            type Element = Context::OrphanElement;

            type ViewState = Context::OrphanViewState;

            fn build(
                &self,
                ctx: &mut Context,
                app_state: &mut State,
            ) -> (Self::Element, Self::ViewState) {
                Context::orphan_build(self, ctx, app_state)
            }

            fn rebuild(
                &self,
                prev: &Self,
                view_state: &mut Self::ViewState,
                ctx: &mut Context,
                element: Mut<'_, Self::Element>,
                app_state: &mut State,
            ) {
                Context::orphan_rebuild(self, prev, view_state, ctx, element, app_state);
            }

            fn teardown(
                &self,
                view_state: &mut Self::ViewState,
                ctx: &mut Context,
                element: Mut<'_, Self::Element>,
                app_state: &mut State,
            ) {
                Context::orphan_teardown(self, view_state, ctx, element, app_state);
            }

            fn message(
                &self,
                view_state: &mut Self::ViewState,
                id_path: &[ViewId],
                message: DynMessage,
                app_state: &mut State,
            ) -> MessageResult<Action> {
                Context::orphan_message(self, view_state, id_path, message, app_state)
            }
        }
    };
}

// string impls - should be used for immutable strings which can be selected within and copied from
impl_orphan_view_for!(&'static str);
impl_orphan_view_for!(alloc::string::String);
impl_orphan_view_for!(alloc::borrow::Cow<'static, str>);

// number impls
impl_orphan_view_for!(f32);
impl_orphan_view_for!(f64);
impl_orphan_view_for!(i8);
impl_orphan_view_for!(u8);
impl_orphan_view_for!(i16);
impl_orphan_view_for!(u16);
impl_orphan_view_for!(i32);
impl_orphan_view_for!(u32);
impl_orphan_view_for!(i64);
impl_orphan_view_for!(u64);
impl_orphan_view_for!(u128);
impl_orphan_view_for!(isize);
impl_orphan_view_for!(usize);

#[cfg(feature = "kurbo")]
/// These [`OrphanView`] implementations can e.g. be used in a vector graphics context, as for example seen in `xilem_web` within svg nodes
mod kurbo {
    use super::OrphanView;
    use crate::{DynMessage, MessageResult, Mut, View, ViewId, ViewMarker};
    impl_orphan_view_for!(kurbo::PathSeg);
    impl_orphan_view_for!(kurbo::Arc);
    impl_orphan_view_for!(kurbo::BezPath);
    impl_orphan_view_for!(kurbo::Circle);
    impl_orphan_view_for!(kurbo::CircleSegment);
    impl_orphan_view_for!(kurbo::CubicBez);
    impl_orphan_view_for!(kurbo::Ellipse);
    impl_orphan_view_for!(kurbo::Line);
    impl_orphan_view_for!(kurbo::QuadBez);
    impl_orphan_view_for!(kurbo::Rect);
    impl_orphan_view_for!(kurbo::RoundedRect);
}
