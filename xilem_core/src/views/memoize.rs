// Copyright 2024 the Xilem Authors
// SPDX-License-Identifier: Apache-2.0

use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use crate::{DynMessage, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};

/// A view which supports Memoization.
///
/// The story of Memoization in Xilem is still being worked out,
/// so the details of this view might change.
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct Memoize<Data, InitView, State, Action, Context> {
    data: Data,
    init_view: InitView,
    phantom: PhantomData<fn() -> (State, Action, Context)>,
}

impl<Data, InitView, State, Action, Context> Debug
    for Memoize<Data, InitView, State, Action, Context>
where
    Data: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Memoize")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

const NON_CAPTURING_CLOSURE: &str = "
It's not possible to use function pointers or captured context in closures,
as this potentially messes up the logic of memoize or produces unwanted effects.

For example a different kind of view could be instantiated with a different callback, while the old one is still memoized, but it's not updated then.
It's not possible in Rust currently to check whether the (content of the) callback has changed with the `Fn` trait, which would make this otherwise possible.
";

/// Memoize the view, until the `data` changes (in which case `view` is called again)
///
/// # Examples
///
/// (From the Xilem implementation)
///
/// ```ignore
/// fn memoized_button(count: u32) -> impl WidgetView<i32> {
///     memoize(
///         count, // if this changes, the closure below is reevaluated
///         |count| button(format!("clicked {count} times"), |count| { count += 1; }),
///     )
/// }
/// ```
pub fn memoize<State, Action, Context, Data, V, InitView>(
    data: Data,
    init_view: InitView,
) -> Memoize<Data, InitView, State, Action, Context>
where
    Data: PartialEq + 'static,
    // TODO(DJMcNab): Also accept `&mut State` in this argument closure
    InitView: Fn(&Data) -> V + 'static,
    V: View<State, Action, Context>,
    Context: ViewPathTracker,
{
    const {
        assert!(size_of::<InitView>() == 0, "{}", NON_CAPTURING_CLOSURE);
    }
    Memoize {
        data,
        init_view,
        phantom: PhantomData,
    }
}

#[allow(unnameable_types)] // reason: Implementation detail, public because of trait visibility rules
#[derive(Debug)]
pub struct MemoizeState<V, VState> {
    view: V,
    view_state: VState,
    dirty: bool,
}

impl<Data, ViewFn, State, Action, Context> ViewMarker
    for Memoize<Data, ViewFn, State, Action, Context>
{
}
impl<State, Action, Context, Data, V, ViewFn> View<State, Action, Context>
    for Memoize<Data, ViewFn, State, Action, Context>
where
    State: 'static,
    Action: 'static,
    Context: ViewPathTracker + 'static,
    Data: PartialEq + 'static,
    V: View<State, Action, Context>,
    ViewFn: Fn(&Data) -> V + 'static,
{
    type ViewState = MemoizeState<V, V::ViewState>;

    type Element = V::Element;

    fn build(&self, ctx: &mut Context, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let view = (self.init_view)(&self.data);
        let (element, view_state) = view.build(ctx, app_state);
        let memoize_state = MemoizeState {
            view,
            view_state,
            dirty: false,
        };
        (element, memoize_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut Context,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if core::mem::take(&mut view_state.dirty) || prev.data != self.data {
            let view = (self.init_view)(&self.data);
            view.rebuild(
                &view_state.view,
                &mut view_state.view_state,
                ctx,
                element,
                app_state,
            );
            view_state.view = view;
        }
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        id_path: &[ViewId],
        message: DynMessage,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let message_result =
            view_state
                .view
                .message(&mut view_state.view_state, id_path, message, app_state);
        if matches!(message_result, MessageResult::RequestRebuild) {
            view_state.dirty = true;
        }
        message_result
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut Context,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        view_state
            .view
            .teardown(&mut view_state.view_state, ctx, element, app_state);
    }
}

/// This view can be used, when there's no access to the `State`, other than in event callbacks
pub struct Frozen<InitView, State, Action> {
    init_view: InitView,
    phantom: PhantomData<fn() -> (State, Action)>,
}

impl<InitView, State, Action> Debug for Frozen<InitView, State, Action> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Frozen").finish_non_exhaustive()
    }
}

/// This view can be used, when the view returned by `init_view` doesn't access the `State`, other than in event callbacks
/// It only evaluates the `init_view` once, when it's being created.
///
/// # Examples
///
/// (From the Xilem implementation)
///
/// ```ignore
/// fn frozen_button() -> impl WidgetView<i32> {
///     frozen(|| button("doesn't access any external state", |count| { count += 1; })),
/// }
/// ```
pub fn frozen<State, Action, Context, V, InitView>(
    init_view: InitView,
) -> Frozen<InitView, State, Action>
where
    State: 'static,
    Action: 'static,
    Context: ViewPathTracker,
    V: View<State, Action, Context>,
    InitView: Fn() -> V,
{
    const {
        assert!(size_of::<InitView>() == 0, "{}", NON_CAPTURING_CLOSURE);
    }
    Frozen {
        init_view,
        phantom: PhantomData,
    }
}

impl<InitView, State, Action> ViewMarker for Frozen<InitView, State, Action> {}
impl<State, Action, Context, V, InitView> View<State, Action, Context>
    for Frozen<InitView, State, Action>
where
    State: 'static,
    Action: 'static,
    Context: ViewPathTracker,
    V: View<State, Action, Context>,
    InitView: Fn() -> V + 'static,
{
    type Element = V::Element;

    type ViewState = MemoizeState<V, V::ViewState>;

    fn build(&self, ctx: &mut Context, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let view = (self.init_view)();
        let (element, view_state) = view.build(ctx, app_state);
        let memoize_state = MemoizeState {
            view,
            view_state,
            dirty: false,
        };
        (element, memoize_state)
    }

    fn rebuild(
        &self,
        _prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut Context,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if core::mem::take(&mut view_state.dirty) {
            let view = (self.init_view)();
            view_state.view.rebuild(
                &view_state.view,
                &mut view_state.view_state,
                ctx,
                element,
                app_state,
            );
            view_state.view = view;
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut Context,
        element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        view_state
            .view
            .teardown(&mut view_state.view_state, ctx, element, app_state);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        id_path: &[ViewId],
        message: DynMessage,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        let message_result =
            view_state
                .view
                .message(&mut view_state.view_state, id_path, message, app_state);
        if matches!(message_result, MessageResult::RequestRebuild) {
            view_state.dirty = true;
        }
        message_result
    }
}
