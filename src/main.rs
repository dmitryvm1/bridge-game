#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use bridge_game::game::{Card, Game, Phase, Suit};
use gpui::{
    App, Bounds, Context, ElementId, IntoElement, QuitMode, SharedString, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use std::time::Duration;

const BG: u32 = 0x101a19;
const PANEL: u32 = 0x182724;
const MUTED: u32 = 0x9bb2a8;
const GOLD: u32 = 0xe4c586;

struct Bridge {
    game: Game,
    selected: Vec<usize>,
    chosen: Suit,
    count: usize,
    queen: bool,
    started: bool,
    rules: bool,
    message: String,
    thinking: bool,
}
impl Bridge {
    fn new() -> Self {
        Self {
            game: Game::new(3, false),
            selected: vec![],
            chosen: Suit::Hearts,
            count: 3,
            queen: false,
            started: false,
            rules: false,
            message: String::new(),
            thinking: false,
        }
    }
    fn start(&mut self, cx: &mut Context<Self>) {
        self.game = Game::new(self.count, self.queen);
        self.selected.clear();
        self.message.clear();
        self.started = true;
        self.schedule(cx);
        cx.notify();
    }
    fn schedule(&mut self, cx: &mut Context<Self>) {
        if self.thinking
            || !self.started
            || self.game.phase != Phase::Playing
            || self.game.turn == 0
        {
            return;
        }
        self.thinking = true;
        cx.spawn(async |view, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(750))
                .await;
            let _ = view.update(cx, |this, cx| {
                this.thinking = false;
                if this.started {
                    this.game.bot_step();
                    this.schedule(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        active: bool,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id)
            .px_3()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(rgb(if active { GOLD } else { 0x3a5047 }))
            .bg(rgb(if active { 0x3f4935 } else { PANEL }))
            .text_color(rgb(if active { GOLD } else { 0xdce6df }))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x34483c)))
            .child(label.into())
    }
    fn card(card: Card, selected: bool, legal: bool) -> gpui::Div {
        div()
            .w(px(60.))
            .h(px(84.))
            .flex_none()
            .flex()
            .flex_col()
            .justify_between()
            .p_1()
            .rounded_lg()
            .border_2()
            .border_color(rgb(if selected { GOLD } else { 0xe0e3d8 }))
            .bg(rgb(if selected {
                0xffefd0
            } else if legal {
                0xf6f3e9
            } else {
                0xc3cbc0
            }))
            .text_color(rgb(if card.suit.red() { 0xac3f46 } else { 0x203d35 }))
            .text_sm()
            .child(card.label())
            .child(div().text_2xl().self_center().child(card.suit.symbol()))
            .child(div().self_end().text_sm().child(card.label()))
    }
    fn setup(&self, cx: &mut Context<Self>) -> gpui::Div {
        div().flex().flex_col().gap_3().max_w(px(640.)).w_full().p_3().rounded_xl().bg(rgb(PANEL)).border_1().border_color(rgb(0x34483c))
            .child(div().text_color(rgb(GOLD)).text_sm().child("36 CARDS · 2–4 PLAYERS · ONE LAST CARD"))
            .child(div().text_3xl().child("A seat at the table."))
            .child(div().text_color(rgb(MUTED)).child("Match suits or ranks. Change the game with special cards. Empty your hand, keep your score low, and stay below 126."))
            .child(div().child("Players, including you"))
            .child(div().flex().gap_3().children((2..=4).map(|n| self.button(("players", n), format!("{n} players"), self.count == n).on_click(cx.listener(move |this, _, _, cx| { this.count = n; cx.notify(); })))))
            .child(self.button("queen", if self.queen { "♠ Queen of Spades: on · draw 5 and skip" } else { "♠ Queen of Spades: off" }, self.queen).on_click(cx.listener(|this, _, _, cx| { this.queen = !this.queen; cx.notify(); })))
            .child(div().text_sm().text_color(rgb(MUTED)).child("You play against computer opponents. The dealer rotates each round. Exactly 125 resets your score to zero."))
            .child(self.button("start", "Take your seat  →", true).on_click(cx.listener(|this, _, _, cx| this.start(cx))))
    }
    fn table(&self, cx: &mut Context<Self>) -> gpui::Div {
        let human = self.game.phase == Phase::Playing && self.game.turn == 0;
        let status = match self.game.phase {
            Phase::Playing if human => "Your turn".into(),
            Phase::Playing => format!("{} is thinking…", self.game.players[self.game.turn].name),
            Phase::RoundOver(w) => format!("{} wins the round", self.game.players[w].name),
            Phase::GameOver(l) => format!("Game over · {} exceeded 125", self.game.players[l].name),
        };
        let mut field = div().flex().flex_col().flex_1().gap_3().min_w_0()
            .child(div().flex().gap_3().children(self.game.players.iter().enumerate().skip(1).map(|(i,p)| {
                div().flex_1().rounded_lg().p_3().border_1().border_color(rgb(if self.game.turn == i && self.game.phase == Phase::Playing { GOLD } else { 0x34483c })).bg(rgb(PANEL))
                    .child(div().text_color(rgb(MUTED)).text_sm().child(p.name.clone()))
                    .child(div().mt_1().text_lg().child(format!("{} cards   ·   {} points",p.hand.len(),p.score)))
                    .child(div().mt_1().text_color(rgb(0x608474)).child("▧ ".repeat(p.hand.len().min(12))))
            })))
            .child(div().flex().flex_col().items_center().justify_center().gap_2().py_3().rounded_xl().bg(rgb(0x1c3b30)).border_1().border_color(rgb(0x365544))
                .child(div().text_color(rgb(GOLD)).text_sm().child(format!("ROUND {}   /   {} DEALS",self.game.round,self.game.players[self.game.dealer].name.to_uppercase())))
                .child(div().flex().gap_6().items_center()
                    .child(div().flex().flex_col().items_center().gap_2()
                        .child(div().w(px(60.)).h(px(84.)).rounded_lg().border_2().border_color(rgb(0x8da58d)).bg(rgb(0x142b25)).flex().items_center().justify_center().text_3xl().text_color(rgb(GOLD)).child("▧"))
                        .child(div().text_sm().text_color(rgb(MUTED)).child(format!("{} in deck",self.game.deck.len()))))
                    .child(div().flex().flex_col().items_center().gap_2().child(Self::card(self.game.top(),false,true)).child(div().text_sm().text_color(rgb(MUTED)).child("Discard"))))
                .child(div().text_lg().child(status))
                .child(div().text_sm().text_color(rgb(MUTED)).child(format!("Follow {} or rank · Jacks are wild",self.game.suit.symbol()))))
            .child(div().flex().justify_between().child(div().text_lg().child("Your hand")).child(div().text_color(rgb(GOLD)).child(format!("{} points",self.game.players[0].score))))
            .child(div().id("hand").flex().flex_wrap().gap_2().max_h(px(176.)).overflow_y_scroll().children(self.game.players[0].hand.iter().enumerate().map(|(i,&card)| {
                div().id(("card",i)).cursor_pointer().child(Self::card(card,self.selected.contains(&i),human && self.game.legal(card)))
                    .on_click(cx.listener(move |this,_,_,cx| { if this.game.turn == 0 && this.game.phase == Phase::Playing {
                        if let Some(pos)=this.selected.iter().position(|&n|n==i) { this.selected.remove(pos); }
                        else { if this.selected.first().is_some_and(|&n|this.game.players[0].hand[n].rank != card.rank) {this.selected.clear();} this.selected.push(i); }
                        this.message.clear();cx.notify();
                    }}))
            })))
            .child(div().text_sm().text_color(rgb(MUTED)).child("Click cards to select. For a group, select a legal lead card first, then cards of the same rank."));
        if self.selected.iter().any(|&i| {
            self.game.players[0]
                .hand
                .get(i)
                .is_some_and(|c| c.rank == 11)
        }) {
            field = field.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("Choose a suit:")
                    .children(Suit::ALL.into_iter().enumerate().map(|(i, s)| {
                        self.button(("suit", i), s.symbol(), s == self.chosen)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.chosen = s;
                                cx.notify();
                            }))
                    })),
            );
        }
        field = field.child(
            div()
                .flex()
                .gap_3()
                .child(
                    self.button("play", "Play selected", human && !self.selected.is_empty())
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.game.turn == 0 && this.game.phase == Phase::Playing {
                                match this.game.play(&this.selected, this.chosen) {
                                    Ok(()) => {
                                        this.selected.clear();
                                        this.message.clear();
                                        this.schedule(cx);
                                    }
                                    Err(e) => this.message = e,
                                }
                                cx.notify();
                            }
                        })),
                )
                .child(
                    self.button("draw", "Draw cards", human && !self.game.can_play())
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.game.turn == 0 && this.game.phase == Phase::Playing {
                                match this.game.draw() {
                                    Ok(()) => {
                                        this.selected.clear();
                                        this.message.clear();
                                        this.schedule(cx);
                                    }
                                    Err(e) => this.message = e,
                                }
                                cx.notify();
                            }
                        })),
                ),
        );
        if !self.message.is_empty() {
            field = field.child(div().text_color(rgb(GOLD)).child(self.message.clone()));
        }
        if self.game.phase != Phase::Playing {
            field = field.child(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(rgb(PANEL))
                    .flex()
                    .flex_col()
                    .gap_3()
                    .children(self.game.players.iter().enumerate().map(|(i, p)| {
                        div().child(format!(
                            "{}   {:+} this round   →   {} total",
                            p.name, self.game.last_deltas[i], p.score
                        ))
                    }))
                    .child(
                        self.button(
                            "continue",
                            if matches!(self.game.phase, Phase::RoundOver(_)) {
                                "Deal next round →"
                            } else {
                                "Play a new game →"
                            },
                            true,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            if matches!(this.game.phase, Phase::RoundOver(_)) {
                                this.game.next_round();
                                this.selected.clear();
                                this.message.clear();
                                this.schedule(cx);
                            } else {
                                this.start(cx);
                            }
                            cx.notify();
                        })),
                    ),
            );
        }
        div().flex().gap_4().items_start().child(field).child(
            div()
                .id("history")
                .max_h(px(450.))
                .overflow_y_scroll()
                .w(px(200.))
                .flex_none()
                .p_3()
                .rounded_lg()
                .bg(rgb(PANEL))
                .flex()
                .flex_col()
                .gap_3()
                .child(div().text_color(rgb(GOLD)).text_sm().child("AT THE TABLE"))
                .children(self.game.log.iter().rev().take(12).map(|line| {
                    div()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .pb_2()
                        .border_b_1()
                        .border_color(rgb(0x2b4036))
                        .child(line.clone())
                })),
        )
    }
}
impl Render for Bridge {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(BG)).text_color(rgb(0xf0f2e9)).font_family("sans-serif").text_sm().flex().flex_col()
            .child(div().flex().items_center().justify_between().px_4().py_2().border_b_1().border_color(rgb(0x2b4036))
                .child(div().flex().items_center().gap_3().child(div().text_3xl().text_color(rgb(GOLD)).child("♣")).child(div().text_xl().child("BRIDGE")).child(div().text_sm().text_color(rgb(MUTED)).child("THE 36-CARD GAME")))
                .child(div().flex().gap_3()
                    .child(self.button("rules","How to play",self.rules).on_click(cx.listener(|this,_,_,cx|{this.rules=!this.rules;cx.notify();})))
                    .child(self.button("new","New game",false).on_click(cx.listener(|this,_,_,cx|{this.started=false;this.selected.clear();cx.notify();})))))
            .child(div().id("content").flex_1().min_h_0().overflow_y_scroll().p_3()
                .child(if self.rules {div().max_w(px(850.)).flex().flex_col().gap_4().child(div().text_2xl().text_color(rgb(GOLD)).child("How to play")).child(div().text_sm().child(include_str!("../DESIGN.md"))).into_any_element()}
                    else if self.started {self.table(cx).into_any_element()}
                    else {div().size_full().flex().items_center().justify_center().child(self.setup(cx)).into_any_element()}))
            .child(div().px_4().py_2().text_sm().text_color(rgb(MUTED)).border_t_1().border_color(rgb(0x2b4036)).child("6  Cover it    ·    7  Draw 1    ·    8  Draw 2 + skip    ·    J  Choose suit    ·    A  Skip    |    Finish with J: −20 each"))
    }
}
fn main() {
    gpui_platform::application()
        .with_quit_mode(QuitMode::LastWindowClosed)
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(1100.), px(680.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(800.), px(560.))),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Bridge · The 36-card game".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Bridge::new()),
            )
            .expect("Unable to open the game window");
            cx.activate(true);
        });
}
