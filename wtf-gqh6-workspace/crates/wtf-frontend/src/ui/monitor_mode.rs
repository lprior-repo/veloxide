#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod calc;
mod types;

pub use calc::{
    compute_playback_interval, format_seq_label, format_timestamp_relative, should_disable_sse,
    validate_replay_seq,
};
pub use types::{
    FrozenState, MonitorMode, ReplayResponse, ScrubberBounds, ScrubberError, ScrubberState,
};

use crate::ui::time_travel::StepIndex;
use dioxus::prelude::*;

#[component]
pub fn TimeTravelScrubber(
    instance_id: ReadSignal<Option<String>>,
    max_seq: ReadSignal<u64>,
    on_replay_request: EventHandler<u64>,
    on_reset_to_live: EventHandler<()>,
    on_sse_disable: EventHandler<bool>,
) -> Element {
    let mut scrubber_state: Signal<Option<ScrubberState>> = use_signal(|| None);
    let mut current_seq: Signal<u64> = use_signal(|| 0);
    let mut is_playing: Signal<bool> = use_signal(|| false);

    let mode = (*scrubber_state.read()).map_or(MonitorMode::Live, |s| s.mode.clone());
    let is_historical = mode.is_historical();
    let bounds = ScrubberBounds::new(0, *max_seq.read());

    let effective_max = if bounds.max_seq == 0 {
        1
    } else {
        bounds.max_seq
    };

    let on_slider_change = move |ev: Event<FormData>| {
        let value = ev.value().parse::<u64>().unwrap_or(0);
        let clamped = bounds.clamp(value);
        current_seq.set(clamped);
    };

    let on_replay_click = move |_| {
        let seq = *current_seq.read();
        if bounds.contains(seq) {
            on_replay_request.call(seq);
        }
    };

    let on_play_click = move |_| {
        let currently_playing = *is_playing.read();
        is_playing.set(!currently_playing);
        if !currently_playing && !is_historical {
            let seq = *current_seq.read();
            let max = *max_seq.read();
            if seq < max {
                on_replay_request.call(seq);
            }
        }
    };

    let on_reset_click = move |_| {
        is_playing.set(false);
        scrubber_state.set(None);
        current_seq.set(0);
        on_reset_to_live.call(());
        on_sse_disable.call(false);
    };

    let tick_count = 5;
    let tick_marks = (0..=tick_count)
        .map(|i| {
            let seq = (effective_max * i) / tick_count;
            seq
        })
        .collect::<Vec<_>>();

    rsx! {
        div {
            class: "flex flex-col gap-2 p-3 bg-white border-b border-slate-200",

            div {
                class: "flex items-center justify-between",

                div { class: "flex items-center gap-2",
                    if is_historical {
                        div { class: "w-2 h-2 rounded-full bg-indigo-500" }
                        span { class: "text-[11px] text-indigo-700 font-medium", "Historical Mode" }
                        span { class: "text-[10px] text-slate-500", "SSE disabled" }
                    } else {
                        div { class: "w-2 h-2 rounded-full bg-emerald-500 animate-pulse" }
                        span { class: "text-[11px] text-emerald-700 font-medium", "Live Mode" }
                    }
                }

                div { class: "flex items-center gap-2",
                    span { class: "text-[11px] font-mono text-slate-600",
                        "seq: {current_seq.read()} / {effective_max}"
                    }
                }
            }

            div { class: "relative",
                input {
                    r#type: "range",
                    class: "w-full h-2 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-indigo-600",
                    min: "0",
                    max: "{effective_max}",
                    value: "{current_seq.read()}",
                    oninput: on_slider_change,
                }

                div { class: "flex justify-between mt-1",
                    for tick in tick_marks.iter() {
                        span { class: "text-[9px] text-slate-400", "{tick}" }
                    }
                }
            }

            div { class: "flex items-center justify-between",
                button {
                    class: "px-3 py-1.5 text-[11px] font-medium rounded border transition-colors
                        if is_historical {
                            "bg-indigo-50 text-indigo-700 border-indigo-300 hover:bg-indigo-100"
                        } else {
                            "bg-slate-50 text-slate-600 border-slate-300 hover:bg-slate-100"
                        }",
                    onclick: on_replay_click,
                    disabled: bounds.max_seq == 0,
                    "Go to seq"
                }

                button {
                    class: "px-3 py-1.5 text-[11px] font-medium rounded border transition-colors
                        if *is_playing.read() {
                            "bg-amber-50 text-amber-700 border-amber-300 hover:bg-amber-100"
                        } else {
                            "bg-slate-50 text-slate-600 border-slate-300 hover:bg-slate-100"
                        }",
                    onclick: on_play_click,
                    disabled: bounds.max_seq == 0 || (is_historical && *current_seq.read() >= *max_seq.read()),
                    if *is_playing.read() {
                        "Pause"
                    } else {
                        "Play"
                    }
                }

                button {
                    class: "px-3 py-1.5 text-[11px] font-medium rounded border transition-colors
                        if is_historical {
                            "bg-emerald-50 text-emerald-700 border-emerald-300 hover:bg-emerald-100"
                        } else {
                            "bg-slate-50 text-slate-400 border-slate-200 cursor-not-allowed"
                        }",
                    onclick: on_reset_click,
                    disabled: !is_historical,
                    "Reset to Live"
                }
            }

            if is_historical {
                div { class: "mt-2 p-2 bg-slate-50 rounded border border-slate-200",
                    if let Some(state) = scrubber_state.read().as_ref() {
                        div { class: "text-[10px] text-slate-600",
                            "Historical state at seq "
                            span { class: "font-mono font-medium", "{state.seq}" }
                            " — "
                            span { class: "text-slate-500", "{format_timestamp_relative(&state.frozen_state.timestamp)}" }
                        }
                    }
                }
            }
        }
    }
}
