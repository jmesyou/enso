//! Events implementation. Events behave in a similar way to JavaScript Events. When an event is
//! emitted, it is propagated in three stages: capturing, target, and bubbling. Each stage is
//! configurable and some events propagation can be cancelled. To learn more about the mechanics,
//! see: https://javascript.info/bubbling-and-capturing.

use crate::prelude::*;

use crate::display::object::instance::Instance;
use crate::display::object::instance::WeakInstance;



// =============
// === State ===
// =============

/// Event state. It can be used to determine whether the event is being propagated, its propagation
/// is cancelled, or that the propagation cannot be cancelled. See docs of this module to learn
/// more.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Running,
    RunningNonCancellable,
    Cancelled,
}



// =================
// === SomeEvent ===
// =================

/// Similar to [`Event`] but with a hidden payload type. It is used to construct, configure, and
/// emit new events.
#[allow(missing_docs)]
#[derive(Clone, CloneRef, Debug)]
pub struct SomeEvent {
    pub data:     frp::AnyData,
    state:        Rc<Cell<State>>,
    /// Indicates whether the event participates in the capturing phase.
    pub captures: Rc<Cell<bool>>,
    /// Indicates whether the event participates in the bubbling phase.
    pub bubbles:  Rc<Cell<bool>>,
}

impl SomeEvent {
    /// Constructor.
    pub fn new<T: 'static>(target: Option<WeakInstance>, payload: T) -> Self {
        let event = Event::new(target, payload);
        let state = event.state.clone_ref();
        let captures = Rc::new(Cell::new(true));
        let bubbles = Rc::new(Cell::new(true));
        Self { data: frp::AnyData::new(event), state, captures, bubbles }
    }

    /// The [`State]` of the event.
    pub fn state(&self) -> State {
        self.state.get()
    }

    /// Check whether the event was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.state() == State::Cancelled
    }
}

impl Default for SomeEvent {
    fn default() -> Self {
        Self::new::<()>(None, ())
    }
}



// =============
// === Event ===
// =============

/// The [`Event`] interface represents an event which takes place in the EnsoGL display object
/// hierarchy.
///
/// An event can be triggered by the user action e.g. clicking the mouse button or tapping keyboard,
/// or generated by APIs to represent the progress of an asynchronous task. It can also be triggered
/// programmatically, such as by calling the [`display::object::Instance::focus()`] method of an
/// element, or by defining the event, then sending it to a specified target using
/// [`display::object::Instance::event_source::emit(...)`].
///
/// See the JavaScript counterpart of this struct:
/// https://developer.mozilla.org/en-US/docs/Web/API/Event.
#[derive(Derivative, Deref)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound = "T: Debug"))]
#[derivative(Default(bound = "T: Default"))]
pub struct Event<T> {
    data: Rc<EventData<T>>,
}

/// Internal representation of [`Event`].
#[derive(Deref, Derivative)]
#[derivative(Debug(bound = "T: Debug"))]
#[derivative(Default(bound = "T: Default"))]
pub struct EventData<T> {
    #[deref]
    payload: T,
    target:  Option<WeakInstance>,
    state:   Rc<Cell<State>>,
}

impl<T> Event<T> {
    fn new(target: Option<WeakInstance>, payload: T) -> Self {
        let state = default();
        let data = Rc::new(EventData { payload, target, state });
        Self { data }
    }

    /// Prevents further propagation of the current event in the capturing and bubbling phases. It
    /// also does NOT prevent immediate propagation to other event-handlers.
    ///
    /// See: https://developer.mozilla.org/en-US/docs/Web/API/Event/stopPropagation.
    pub fn stop_propagation(&self) {
        if self.state.get() == State::RunningNonCancellable {
            warn!("Trying to cancel a non-cancellable event.");
        } else {
            self.state.set(State::Cancelled);
        }
    }

    /// A reference to the object onto which the event was dispatched.
    ///
    /// See: https://developer.mozilla.org/en-US/docs/Web/API/Event/target.
    pub fn target(&self) -> Option<Instance> {
        self.data.target.as_ref().and_then(|t| t.upgrade())
    }
}



// ====================
// === Basic Events ===
// ====================

/// The [`Focus`] event fires when an element has received focus. The event does not bubble, but the
/// related [`FocusIn`] event that follows does bubble.
///
/// The opposite of [`Focus`] is the [`Blur`] event, which fires when the element has lost focus.
///
/// The [`Focus`] event is not cancelable.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/focus_event.
#[derive(Clone, Copy, Debug, Default)]
pub struct Focus;

/// The [`Blur`] event fires when an element has lost focus. The event does not bubble, but the
/// related [`FocusOut`] event that follows does bubble.
///
/// The opposite of [`Blur`] is the [Focus] event, which fires when the element has received focus.
/// The [`Blur`] event is not cancelable.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/blur_event.
#[derive(Clone, Copy, Debug, Default)]
pub struct Blur;

/// The [`FocusIn`] event fires when an element has received focus, after the [`Focus`] event. The
/// two events differ in that [`FocusIn`] bubbles, while [`Focus`] does not.
///
/// The opposite of [`FocusIn`] is the [`FocusOut`] event, which fires when the element has lost
/// focus. The [`FocusIn`] event is not cancelable.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/focusin_event.
#[derive(Clone, Copy, Debug, Default)]
pub struct FocusIn;

/// The [`FocusOut`] event fires when an element has lost focus, after the [`Blur`] event. The two
/// events differ in that [`FocusOut`] bubbles, while [`Blur`] does not.
///
/// The opposite of [`FocusOut`] is the [`FocusIn`] event, which fires when the element has received
/// focus. The [`FocusOut`] event is not cancelable.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/Element/focusout_event.
#[derive(Clone, Copy, Debug, Default)]
pub struct FocusOut;
