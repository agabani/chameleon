use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct Props {
    pub name: AttrValue,
    pub secret_words: Vec<AttrValue>,
}

pub struct TopicCard {}

impl Component for TopicCard {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = &ctx.props().name;
        let secret_words =
            ctx.props()
                .secret_words
                .iter()
                .enumerate()
                .map(|(index, secret_word)| {
                    let column = index % 4;
                    let row = index / 4;
                    (index, secret_word, row, column)
                });

        html! {
            <div class="topic-card">
                <div class="name">{ name }</div>
                <div></div>
                <div class="coordinate letter">{ "A" }</div>
                <div class="coordinate letter">{ "B" }</div>
                <div class="coordinate letter">{ "C" }</div>
                <div class="coordinate letter">{ "D" }</div>
                {
                    secret_words.map(|(index, secret_word, row, column)| {
                        html! {
                            <>
                                if column == 0 {
                                    <div class="coordinate number">{ row + 1 }</div>
                                }
                                <div class={if (index - row) % 2 == 0 { "secret-word blue" } else { "secret-word" }}>
                                    { secret_word }
                                </div>
                            </>
                        }
                    }).collect::<Html>()
                }
            </div>
        }
    }
}
