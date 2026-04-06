use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    ecs::{
        component::Component,
        query::With,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, ResMut},
    },
    prelude::Res,
    time::{Time, Timer},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Henrik Stenslie".to_string())));
    commands.spawn((Person, Name("Sander Henriksen".to_string())));
    commands.spawn((Person, Name("Albert Engan".to_string())));
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0)
        }
    }
}
fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Henrik Stenslie" {
            name.0 = "Oda Stenslie".to_string();
            break;
        }
    }
}

pub struct HelloPlugin;
impl bevy::app::Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_people)
            .insert_resource(GreetTimer(Timer::from_seconds(
                2.0,
                bevy::time::TimerMode::Repeating,
            )))
            .add_systems(Update, (greet_people, update_people.chain()));
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);
