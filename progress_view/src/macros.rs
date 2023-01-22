pub macro make_tasks {
    // empty: we've reached the end
    ($app:ident;) => {

    },
    // app; task("Lorem ipsum")
    ($app:ident; $indent:expr, $type:ident($($inner:tt)*); $($tail:tt)*) => {
        $app.add_widget(_make_widget!($indent, $type($($inner)*)));

        make_tasks!($app; $($tail)*)
    },
    // app; task("Lorem ipsum") -> |sender| { .. };
    ($app:ident; $indent:expr, $type:ident($($inner:tt)*) -> |$sender:ident| $task:block; $($tail:tt)*) => {
        #[allow(unused_mut)]
        let mut cls = async move |$sender: crate::app::UpdateSender| $task;
        let idx = $app.add_widget(_make_widget!($indent, $type($($inner)*)));
        $app.add_task(cls, idx);

        make_tasks!($app; $($tail)*)
    },
    // app; task("Lorem ipsum") -> |sender| ..;
    ($app:ident; $indent:expr, $type:ident($($inner:tt)*) -> |$sender:ident| $fn:stmt; $($tail:tt)*) => {
        #[allow(unused_mut)]
        let mut cls = async move |$sender| { $fn };
        let idx = $app.add_widget(_make_widget!($indent, $type($($inner)*)));
        $app.add_task(cls, idx);

        make_tasks!($app; $($tail)*)
    },
    // app; task("Lorem ipsum") -> fn_name;
    ($app:ident; $indent:expr, $type:ident($($inner:tt)*) -> $fn_name:ident; $($tail:tt)*) => {
        let idx = $app.add_widget(_make_widget!($indent, $type($($inner)*)));
        $app.add_task($fn_name, idx);

        make_tasks!($app; $($tail)*)
    },
}

macro _make_widget {
    ($indent:expr, task($msg:expr)) => {
        crate::widget::Widget::new_task($msg, $indent)
    },
    ($indent:expr, text($msg:expr)) => {
        crate::widget::Widget::new_text($msg, $indent)
    },
    ($indent:expr, progress($msg:expr, $total:expr)) => {
        crate::widget::Widget::new_progress($msg, $indent, $total)
    },
    ($indent:expr, percentage($msg:expr)) => {
        crate::widget::Widget::new_percentage($msg, $indent)
    },
}

mod tests {

    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::update::Update;
    #[allow(unused_imports)]
    use crate::widget::Widget;

    #[test]
    fn test_make_tasks() {
        use crate::app::{App, UpdateSender};
        use crossterm::cursor::MoveDown;
        use crossterm::execute;
        use std::io::stdout;
        use std::time::Duration;

        let mut app_orig = App::default();
        let mut app_mac = App::default();

        app_orig.add_widgets([
            Widget::new_task("root", 0),
            Widget::new_task("sub 1", 1),
            Widget::new_task("sub 1", 1),
            Widget::new_task("sub 2", 2),
            Widget::new_task("sub 1", 1),
            Widget::new_task("root", 0),
            Widget::new_task("sub 1", 1),
            Widget::new_task("root", 0),
        ]);

        async fn yeet(s: UpdateSender) {
            s.send(Update::set_message("changed")).await;
        }

        async fn yeet_msg(s: UpdateSender, msg: String) {
            s.send(Update::set_message(msg)).await;
        }

        app_orig.add_task(yeet, 1);
        app_orig.add_task(yeet, 2);
        app_orig.add_task(yeet, 3);

        let mut v = "change".to_string();

        make_tasks!(
            app_mac;
            0, task("root");
            1, task("sub 1") -> |s| yeet(s).await;
            1, task("sub 1") -> yeet;
            2, task("sub 2") -> |s| {
                v.push_str("d");
                yeet_msg(s, v).await;
            };
            1, task("sub 1");
            0, task("root");
            1, task("sub 1");
            0, task("root");
        );

        std::thread::sleep(Duration::from_millis(100));

        assert_eq!(app_mac.widgets.len(), app_orig.widgets.len());

        app_orig.render();
        app_mac.render();
        execute!(stdout(), MoveDown(app_mac.widgets.len() as u16)).unwrap();

        assert_eq!(app_mac.widgets[1].message, "changed");
        assert_eq!(app_mac.widgets[2].message, "changed");
        assert_eq!(app_mac.widgets[3].message, "changed");

        for (l, r) in app_orig.widgets.iter().zip(app_mac.widgets) {
            assert_eq!(*l, r);
        }
    }

    #[test]
    fn test_make_widget() {
        assert_eq!(
            _make_widget!(0, task("Lorem ipsum")),
            Widget::new_task("Lorem ipsum", 0)
        );
        assert_eq!(
            _make_widget!(0, text("Lorem ipsum")),
            Widget::new_text("Lorem ipsum", 0)
        );
        assert_eq!(
            _make_widget!(0, progress("Lorem ipsum", 100)),
            Widget::new_progress("Lorem ipsum", 0, 100)
        );
        assert_eq!(
            _make_widget!(0, percentage("Lorem ipsum")),
            Widget::new_percentage("Lorem ipsum", 0)
        );
    }
}
