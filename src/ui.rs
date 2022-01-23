use crate::category;
use iced::button::{self, Button};
use iced::scrollable::{self, Scrollable};
use iced::text_input::{self, TextInput};
use iced::{
    Align, Application, Clipboard, Column, Command, Container, Element, Font, HorizontalAlignment,
    Length, Row, Settings, Text,
};

pub fn run() -> iced::Result {
    App::run(Settings {
        default_font: Some(include_bytes!("../fonts/meiryo.ttc")),
        ..Settings::default()
    })
}

#[derive(Debug)]
enum App {
    Loading,
    Loaded(AppState),
}

#[derive(Debug, Default)]
struct AppState {
    saving: bool,
    dirty: bool,
    category_list_state: CategoryListState,
    save_button: button::State,
}

#[derive(Debug, Clone)]
enum AppMessage {
    Loaded(Result<SavedState, LoadError>),
    Saved(Result<(), SaveError>),
    Save,
    CategoryListMessage(CategoryListMessage),
}

#[derive(Debug, Clone)]
struct StateWith<Entity, State> {
    entity: Entity,
    state: State,
}

#[derive(Debug, Clone)]
pub enum CategoryState {
    Idle {
        edit_button: button::State,
    },
    Editing {
        text_input: text_input::State,
        delete_button: button::State,
    },
}
impl Default for CategoryState {
    fn default() -> Self {
        CategoryState::Idle {
            edit_button: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CategoryMessage {
    Edit,
    Edited(String),
    FinishEdition,
    Delete,
}

type Category = StateWith<crate::category::Category, CategoryState>;
impl Default for Category {
    fn default() -> Self {
        use crate::utils::establish_connection;
        use transaction::with_ctx;
        let initial_name = "新規カテゴリ";
        // let cn = establish_connection();
        // let tx = with_ctx(|ctx| {
        //     use crate::category::{create, NewCategory};
        //     let new_category = NewCategory { name: initial_name };
        //     create(new_category).run(ctx)
        // });
        // let created_category = transaction_diesel_mysql::run(&cn, tx).unwrap();
        let new_category = category::Category {
            id: 0,
            name: initial_name.to_string(),
        };
        Category {
            // entity: created_category,
            entity: new_category,
            state: CategoryState::default(),
        }
    }
}

#[derive(Debug, Default)]
struct CategoryListState {
    scroll: scrollable::State,
    categories: Vec<Category>,
    add_button: button::State,
}

#[derive(Debug, Clone)]
enum CategoryListMessage {
    CreateCategory,
    CategoryMessage(usize, CategoryMessage),
}

// Persistence
#[derive(Debug, Clone)]
struct SavedState {
    categories: Vec<Category>,
}

#[derive(Debug, Clone)]
enum LoadError {
    DatabaseError,
}

#[derive(Debug, Clone)]
enum SaveError {
    DatabaseError,
}

impl Category {
    fn new(entity: category::Category) -> Self {
        Self {
            entity: entity,
            state: CategoryState::default(),
        }
    }
    fn update(&mut self, message: CategoryMessage) {
        match message {
            CategoryMessage::Edit => {
                let text_input = text_input::State::focused();
                // let mut text_input = text_input::State::focused();
                // text_input.select_all(); // TODO: useで解決できなかったから一旦置いておく

                self.state = CategoryState::Editing {
                    text_input,
                    delete_button: button::State::new(),
                };
            }
            CategoryMessage::Edited(new_name) => {
                self.entity.name = new_name;
            }
            CategoryMessage::FinishEdition => {
                if !self.entity.name.is_empty() {
                    self.state = CategoryState::Idle {
                        edit_button: button::State::new(),
                    }
                }
            }
            CategoryMessage::Delete => {}
        }
    }
    fn view(&mut self) -> Element<CategoryMessage> {
        match &mut self.state {
            CategoryState::Idle { edit_button } => Row::new()
                .spacing(20)
                .align_items(Align::Center)
                .push(Text::new(&self.entity.name).width(Length::Fill))
                .push(
                    Button::new(edit_button, edit_icon())
                        .on_press(CategoryMessage::Edit)
                        .padding(10)
                        .style(style::Button::Icon),
                )
                .into(),
            CategoryState::Editing {
                text_input,
                delete_button,
            } => {
                let text_input = TextInput::new(
                    text_input,
                    "Please input...",
                    &self.entity.name,
                    CategoryMessage::Edited,
                )
                .on_submit(CategoryMessage::FinishEdition)
                .padding(10);

                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(text_input)
                    .push(
                        Button::new(
                            delete_button,
                            Row::new()
                                .spacing(10)
                                .push(delete_icon())
                                .push(Text::new("Delete")),
                        )
                        .on_press(CategoryMessage::Delete)
                        .padding(10)
                        .style(style::Button::Destructive),
                    )
                    .into()
            }
        }
    }
}

impl CategoryListState {
    fn update(&mut self, message: CategoryListMessage) {
        match message {
            CategoryListMessage::CreateCategory => {
                self.categories.push(Category::default());
            }
            CategoryListMessage::CategoryMessage(i, CategoryMessage::Delete) => {
                let maybe_state_with_category = self.categories.get(i);
                match maybe_state_with_category {
                    Some(state_with_category) => {
                        // use crate::utils::establish_connection;
                        // use transaction::with_ctx;
                        // let cn = establish_connection();
                        // let tx = with_ctx(|ctx| {
                        //     use crate::category::delete;
                        //     delete(state_with_category.entity.id).run(ctx)
                        // });
                        // transaction_diesel_mysql::run(&cn, tx).unwrap();
                        self.categories.remove(i);
                    }
                    None => {}
                }
            }
            CategoryListMessage::CategoryMessage(i, category_message) => {
                if let Some(category) = self.categories.get_mut(i) {
                    category.update(category_message);
                }
            }
            _ => {}
        }
    }
    fn view(&mut self) -> Element<CategoryListMessage> {
        let CategoryListState {
            scroll,
            categories,
            add_button,
        } = self;

        let categories: Element<_> = categories
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(20), |column, (i, category)| {
                column.push(
                    category
                        .view()
                        .map(move |message| CategoryListMessage::CategoryMessage(i, message)),
                )
            })
            .into();
        let add_button =
            Button::new(add_button, Text::new("Add")).on_press(CategoryListMessage::CreateCategory);

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .push(categories)
            .push(add_button);

        Scrollable::new(scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill))
            .into()
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = AppMessage;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<AppMessage>) {
        (
            App::Loading,
            Command::perform(SavedState::load(), AppMessage::Loaded),
        )
    }

    fn title(&self) -> String {
        let dirty = match self {
            App::Loading => false,
            App::Loaded(state) => state.dirty,
        };

        format!("App{} - Iced", if dirty { "*" } else { "" })
    }

    fn update(&mut self, message: AppMessage, _clipboard: &mut Clipboard) -> Command<AppMessage> {
        match self {
            App::Loading => {
                match message {
                    AppMessage::Loaded(Ok(saved_state)) => {
                        *self = App::Loaded(AppState {
                            category_list_state: CategoryListState {
                                categories: saved_state.categories,
                                ..CategoryListState::default()
                            },
                            ..AppState::default()
                        });
                    }
                    AppMessage::Loaded(Err(_)) => {
                        *self = App::Loaded(AppState::default());
                    }
                    _ => {}
                }

                Command::none()
            }
            App::Loaded(state) => {
                match message {
                    AppMessage::CategoryListMessage(category_list_message) => {
                        state.category_list_state.update(category_list_message);
                    }
                    AppMessage::Save => {
                        state.dirty = true;
                    }
                    AppMessage::Saved(_) => {
                        state.saving = false;
                    }
                    _ => {}
                }

                if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;

                    println!("セーブします。");
                    Command::perform(
                        SavedState {
                            categories: state.category_list_state.categories.clone(),
                        }
                        .save(),
                        AppMessage::Saved,
                    )
                } else {
                    Command::none()
                }
            }
        }
    }

    fn view(&mut self) -> Element<AppMessage> {
        match self {
            App::Loading => loading_message(),
            App::Loaded(AppState {
                category_list_state,
                save_button,
                ..
            }) => {
                let title = Text::new("todos")
                    .width(Length::Fill)
                    .size(100)
                    .color([0.5, 0.5, 0.5])
                    .horizontal_alignment(HorizontalAlignment::Center);

                let save_button =
                    Button::new(save_button, Text::new("Save")).on_press(AppMessage::Save);

                let list: Element<_> = Column::new()
                    .spacing(20)
                    .push(
                        category_list_state
                            .view()
                            .map(move |message| AppMessage::CategoryListMessage(message)),
                    )
                    .into();

                let content = Column::new()
                    .max_width(800)
                    .spacing(20)
                    .push(title)
                    .push(save_button)
                    .push(list);

                Container::new(content)
                    .width(Length::Fill)
                    .center_x()
                    .into()
            }
        }
    }
}

fn loading_message<'a>() -> Element<'a, AppMessage> {
    Container::new(
        Text::new("Loading...")
            .horizontal_alignment(HorizontalAlignment::Center)
            .size(50),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_y()
    .into()
}

// Fonts
const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .size(20)
}

fn edit_icon() -> Text {
    icon('\u{F303}')
}

fn delete_icon() -> Text {
    icon('\u{F1F8}')
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    async fn load() -> Result<SavedState, LoadError> {
        use crate::utils::establish_connection;
        use transaction::with_ctx;

        let cn = establish_connection();
        let tx = with_ctx(|ctx| crate::category::all().run(ctx));
        let saved_state = transaction_diesel_mysql::run(&cn, tx)
            .map_err(|_| LoadError::DatabaseError)
            .map(|categories| SavedState {
                categories: categories
                    .into_iter()
                    .map(|category| Category::new(category))
                    .collect::<Vec<Category>>(),
            });
        saved_state
    }

    async fn save(self) -> Result<(), SaveError> {
        use crate::category;
        use crate::utils::establish_connection;
        use transaction::with_ctx;

        let new_categories: Vec<category::Category> = self
            .categories
            .into_iter()
            // .filter(|state_with_category| state_with_category.entity.id == 0)
            // .map(|state_with_category| NewCategory {
            //     name: state_with_category.entity.name,
            // })
            .map(|state_with_category| state_with_category.entity)
            .collect();
        println!("{}件のnew_categoriesがあります。", new_categories.len());

        let cn = establish_connection();
        let tx = with_ctx(|ctx| {
            category::clear().run(ctx)?;
            let maybe_saved_categories = new_categories
                .iter()
                .map(|new_category| category::create(new_category.clone()).run(ctx))
                .collect::<Vec<Result<category::Category, _>>>();
            println!(
                "{}件のmaybe_saved_categoriesがあります。",
                maybe_saved_categories.len()
            );
            let is_err = maybe_saved_categories
                .iter()
                .any(|maybe_saved_category| maybe_saved_category.is_err());
            if is_err {
                Err(diesel::result::Error::NotFound)
            } else {
                let saved_categories = maybe_saved_categories
                    .into_iter()
                    .map(|saved_category_in_result| saved_category_in_result.unwrap())
                    .collect::<Vec<category::Category>>();
                println!("{}件のsaved_categoriesがあります。", saved_categories.len());
                Ok(saved_categories)
            }
        });
        let result = transaction_diesel_mysql::run(&cn, tx)
            .map_err(|_| SaveError::DatabaseError)
            .map(|_| ());
        result
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        FilterActive,
        FilterSelected,
        Icon,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
                Button::FilterActive => button::Style::default(),
                Button::FilterSelected => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.7))),
                    border_radius: 10.0,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                },
                Button::Icon => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..button::Style::default()
                },
                Button::Destructive => button::Style {
                    background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                    border_radius: 5.0,
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 1.0),
                    ..button::Style::default()
                },
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            button::Style {
                text_color: match self {
                    Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                    Button::FilterActive => Color::from_rgb(0.2, 0.2, 0.7),
                    _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}
