//! Conversion from UIKit events to GPUI input events.

use gpui::{
    BackGestureEvent, BackGesturePhase, Modifiers, PinchEvent, Pixels, Point, ScrollDelta,
    ScrollEvent, ScrollSource, TouchId, TouchPhase, px,
};
use objc2_core_foundation::CGPoint;
use objc2_ui_kit::{
    UIGestureRecognizerState, UIKeyModifierFlags, UIPanGestureRecognizer, UIPinchGestureRecognizer,
    UIRectEdge, UIScreenEdgePanGestureRecognizer, UITouch, UITouchPhase, UIView,
};

/// A release faster than this, in points per second toward going back, commits
/// a back gesture regardless of how far it travelled.
const BACK_COMMIT_VELOCITY: f64 = 300.;

/// Converts a step of the screen-edge back gesture, whose `Started` phase was
/// already delivered when the recognizer asked to begin.
///
/// Like `UINavigationController`, a release commits when the gesture is past
/// halfway without flicking back, or when it flicks forward quickly. Returns
/// `None` for states that carry no step.
pub fn back_event(
    recognizer: &UIScreenEdgePanGestureRecognizer,
    view: &UIView,
    start_position: Point<Pixels>,
) -> Option<BackGestureEvent> {
    // Gestures from the right edge move content leftward toward going back.
    let direction = if recognizer.edges() == UIRectEdge::Right {
        -1.
    } else {
        1.
    };
    let width = view.bounds().size.width.max(1.);
    let travel = recognizer.translationInView(Some(view)).x * direction;
    let progress = (travel / width).clamp(0., 1.);
    let phase = match recognizer.state() {
        UIGestureRecognizerState::Began | UIGestureRecognizerState::Changed => {
            BackGesturePhase::Progressed
        }
        UIGestureRecognizerState::Ended => {
            let velocity = recognizer.velocityInView(Some(view)).x * direction;
            if velocity > BACK_COMMIT_VELOCITY
                || (progress > 0.5 && velocity > -BACK_COMMIT_VELOCITY)
            {
                BackGesturePhase::Committed
            } else {
                BackGesturePhase::Cancelled
            }
        }
        UIGestureRecognizerState::Cancelled | UIGestureRecognizerState::Failed => {
            BackGesturePhase::Cancelled
        }
        _ => return None,
    };
    Some(BackGestureEvent {
        phase,
        start_position,
        progress: progress as f32,
    })
}

pub fn touch_location_in_view(touch: &UITouch, view: &UIView) -> Point<Pixels> {
    let location = touch.locationInView(Some(view));
    Point::new(px(location.x as f32), px(location.y as f32))
}

pub fn touch_phase(touch: &UITouch) -> TouchPhase {
    match touch.phase() {
        UITouchPhase::Began => TouchPhase::Started,
        UITouchPhase::Moved | UITouchPhase::Stationary => TouchPhase::Moved,
        UITouchPhase::Ended => TouchPhase::Ended,
        _ => TouchPhase::Cancelled,
    }
}

pub fn touch_id(touch: &UITouch) -> TouchId {
    TouchId(touch as *const UITouch as usize as u64)
}

/// Converts one step of a trackpad or mouse-wheel scroll gesture.
///
/// The recognizer's translation is reset after each step, so every event carries
/// only the movement since the previous one. UIKit reports the translation in
/// the direction the content should move, which matches GPUI's pixel deltas.
/// Returns `None` for states that carry no scroll.
pub fn scroll_event(recognizer: &UIPanGestureRecognizer, view: &UIView) -> Option<ScrollEvent> {
    let touch_phase = match recognizer.state() {
        UIGestureRecognizerState::Began => TouchPhase::Started,
        UIGestureRecognizerState::Changed => TouchPhase::Moved,
        UIGestureRecognizerState::Ended => TouchPhase::Ended,
        UIGestureRecognizerState::Cancelled | UIGestureRecognizerState::Failed => {
            TouchPhase::Cancelled
        }
        _ => return None,
    };
    let translation = recognizer.translationInView(Some(view));
    recognizer.setTranslation_inView(CGPoint::ZERO, Some(view));
    let location = recognizer.locationInView(Some(view));
    Some(ScrollEvent {
        position: Point::new(px(location.x as f32), px(location.y as f32)),
        delta: ScrollDelta::Pixels(Point::new(
            px(translation.x as f32),
            px(translation.y as f32),
        )),
        modifiers: modifiers(recognizer.modifierFlags()),
        touch_phase,
        source: ScrollSource::Pointer,
    })
}

/// Converts one step of a two-finger pan into a pinch event that does not zoom: its position
/// is the fingers' centroid, and how far that moved since the last event is how far the
/// content pans. With a pinch running at the same time, content zooms about the centroid as
/// it moves with it, as in Freeform. `None` for states that carry no pan.
pub fn two_finger_pan_event(recognizer: &UIPanGestureRecognizer, view: &UIView) -> Option<PinchEvent> {
    let phase = match recognizer.state() {
        UIGestureRecognizerState::Began => TouchPhase::Started,
        UIGestureRecognizerState::Changed => TouchPhase::Moved,
        UIGestureRecognizerState::Ended => TouchPhase::Ended,
        UIGestureRecognizerState::Cancelled | UIGestureRecognizerState::Failed => {
            TouchPhase::Cancelled
        }
        _ => return None,
    };
    let location = recognizer.locationInView(Some(view));
    Some(PinchEvent {
        position: Point::new(px(location.x as f32), px(location.y as f32)),
        delta: 0.,
        modifiers: modifiers(recognizer.modifierFlags()),
        phase,
    })
}

/// Converts one step of a two-finger pinch.
///
/// The recognizer's scale is reset after each step, so every event carries only
/// the zoom since the previous one, as on macOS: 0.1 is 10% larger. Returns
/// `None` for states that carry no pinch.
pub fn pinch_event(recognizer: &UIPinchGestureRecognizer, view: &UIView) -> Option<PinchEvent> {
    let phase = match recognizer.state() {
        UIGestureRecognizerState::Began => TouchPhase::Started,
        UIGestureRecognizerState::Changed => TouchPhase::Moved,
        UIGestureRecognizerState::Ended => TouchPhase::Ended,
        UIGestureRecognizerState::Cancelled | UIGestureRecognizerState::Failed => {
            TouchPhase::Cancelled
        }
        _ => return None,
    };
    let delta = recognizer.scale() as f32 - 1.;
    recognizer.setScale(1.);
    let location = recognizer.locationInView(Some(view));
    Some(PinchEvent {
        position: Point::new(px(location.x as f32), px(location.y as f32)),
        delta: if matches!(phase, TouchPhase::Started | TouchPhase::Moved) {
            delta
        } else {
            0.
        },
        modifiers: modifiers(recognizer.modifierFlags()),
        phase,
    })
}

fn modifiers(flags: UIKeyModifierFlags) -> Modifiers {
    Modifiers {
        control: flags.contains(UIKeyModifierFlags::Control),
        alt: flags.contains(UIKeyModifierFlags::Alternate),
        shift: flags.contains(UIKeyModifierFlags::Shift),
        platform: flags.contains(UIKeyModifierFlags::Command),
        function: false,
    }
}

pub fn touch_force(touch: &UITouch) -> Option<f32> {
    let maximum_force = touch.maximumPossibleForce();
    if maximum_force <= 0.0 {
        return None;
    }
    Some((touch.force() / maximum_force).clamp(0.0, 1.0) as f32)
}
