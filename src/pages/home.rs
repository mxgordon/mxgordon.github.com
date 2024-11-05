use ev::Event;
use ev::KeyboardEvent;
use ev::SubmitEvent;
use html::span;
use html::Input;
use leptos::*;
use leptos::logging::log;

use crate::commands::about::*;
use crate::commands::search::*;
use crate::commands::typewriter::*;
use crate::commands::utils::*;

fn make_prompt() -> HtmlElement<html::Span>{
    view! {
        <span>"user@mxgordon.com> "</span>
    }
}

#[component]
pub fn PromptInput(
    #[prop()] prompt_input: ReadSignal<String>,
    #[prop()] on_submit: Box<dyn Fn(SubmitEvent) + 'static>,
    #[prop()] on_input: Box<dyn Fn(Event) + 'static>,
    #[prop()] on_keydown: Box<dyn Fn(KeyboardEvent) + 'static>,
    #[prop()] autocomplete: ReadSignal<Vec<String>>,
) -> impl IntoView {
    let prompt_ref = create_node_ref::<Input>();

    prompt_ref.on_load(move |e| {
        let _ = e.on_mount(move |e2| {
            e2.focus().unwrap();
        });
    });

    view! {
        <p class="prompt-line" >{make_prompt()}
            <form on:submit=on_submit>
                <input ref=prompt_ref type="text" id="prompt" prop:value=prompt_input on:input=on_input on:keydown=on_keydown spellcheck="false" autocomplete="off" aria-autocomplete="none" />

                <div class="autocomplete-options">
                    <For each=move || autocomplete.get() key=|cmd_str| cmd_str.clone() children=|cmd| view!{<p>{cmd}</p>} />
                </div>
            </form>
        </p>
    }
    
}

#[component]
pub fn Home() -> impl IntoView {
    let (promptInput, writePromptInput) = create_signal("".to_string());
    let (loadingStage, writeLoadingStage) = create_signal(0);
    let (pastCmds, writePastCmds) = create_signal::<Vec<View>>(vec![]);
    let (autocomplete, writeAutoComplete) = create_signal::<Vec<String>>(vec![]);

    let handleKeyDown = move |e: KeyboardEvent| {
        let key = e.key();
        // let new_value = promptInput.get();

        match key.as_str() {
            "Tab" => {
                e.prevent_default();

                let new_value = promptInput.get();
                let potential_commands = search_commands(new_value);

                if potential_commands.len() >= 1 {
                    writePromptInput.set(potential_commands[0].name.to_string());
                }
            },
            _ => {}
        }

        log!("{:?}", key);
    };

    let handleInput = move |e: Event| {
        writePromptInput.set(event_target_value(&e));
        let new_value = promptInput.get();

        writeAutoComplete.set(search_commands(new_value).iter().map(|c| c.name.to_string()).collect());
    };

    let handleSubmit = move |e: SubmitEvent| {
        e.prevent_default();

        let potential_command = get_command(promptInput.get());
        
        if let Some(command) = potential_command {
            writePastCmds.update(|past| {
                past.push(view! {<p class="prompt-line">{make_prompt()}{promptInput.get()}</p>}.into_view());
                past.push((command.function)(promptInput.get(), Box::new(move ||(writeLoadingStage.set(2)))).into_view());
            });
        } else {
            writePastCmds.update(|past| {
                past.push(view! {<p class="prompt-line">{make_prompt()}{promptInput.get()}</p>}.into_view());
                past.push(view! {<CommandNotFound cmd=promptInput.get() on_finished=Box::new(move ||(writeLoadingStage.set(2))) />}.into_view());
            });
        }
        writeLoadingStage.set(1);
        writePromptInput.set("".to_string());
        writeAutoComplete.set(vec![]);
    };

    let s = view!{<span>"intro"</span>};

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>
                <p>"Errors: "</p>
                // Render a list of errors as strings - good for development purposes
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}
                </ul>
            }
        }>
            <div id="App">
                <p class="prompt-line">{make_prompt()}
                <TypeWriter html_to_type=s base_element=span() delay=200 chunk_sz=1 callback=Box::new(move ||(writeLoadingStage.set(1))) /></p>

                <Show when=move || (loadingStage.get() > 0)>
                    <TypeWriter html_to_type=intro_text() callback=Box::new(move ||(writeLoadingStage.set(2))) />
                </Show>

                {move || {log!("{:?}", pastCmds.get()); pastCmds}}
                // {move || {pastCmds.get()}}
                // {pastCmds}

                <Show when=move || (loadingStage.get() > 1)>
                    <PromptInput prompt_input=promptInput on_submit=Box::new(handleSubmit) on_input=Box::new(handleInput) on_keydown=Box::new(handleKeyDown) autocomplete=autocomplete />
                </Show>
            </div>
        </ErrorBoundary>
    }
}
