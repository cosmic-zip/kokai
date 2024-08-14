use futures::pin_mut;
use glib::{clone, Properties};
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use ipfs::Ipfs;
use libadwaita::prelude::*;
use libadwaita::subclass::application_window::AdwApplicationWindowImpl;
use tokio_stream::StreamExt;
use webkit2gtk::prelude::WebViewExt;
use webkit2gtk::{URISchemeRequest, WebView};

use crate::window_util::{
    initial_connect, new_webkit_settings, update_favicon, update_load_progress, update_nav_bar,
    update_title,
};
use crate::{DATA_DIR, VERSION};

pub mod imp {

    use std::cell::{Cell, RefCell};

    use glib::GString;

    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::Window)]
    pub struct Window {
        // Window parameters
        pub(crate) is_private: Cell<bool>,
        pub(crate) initial_url: RefCell<String>,
        // Left header buttons
        pub(crate) nav_entry: gtk::Entry,
        pub(crate) back_button: gtk::Button,
        pub(crate) forward_button: gtk::Button,
        pub(crate) navigation_buttons: gtk::Box,
        pub(crate) add_tab_button: gtk::Button,
        pub(crate) refresh_button: gtk::Button,
        pub(crate) left_header_buttons: gtk::Box,
        // Right header buttons
        pub(crate) overview_button: gtk::Button,
        pub(crate) downloads_button: gtk::Button,
        pub(crate) find_button: gtk::Button,
        pub(crate) ipfs_button: gtk::Button,
        pub(crate) tor_button: gtk::Button,
        pub(crate) menu_button: gtk::Button,
        pub(crate) right_header_buttons: gtk::Box,
        // HeaderBar
        pub(crate) headerbar: libadwaita::HeaderBar,
        // Menu popover
        pub(crate) zoomout_button: gtk::Button,
        pub(crate) zoomin_button: gtk::Button,
        pub(crate) zoom_buttons: gtk::Box,
        pub(crate) zoomreset_button: gtk::Button,
        pub(crate) fullscreen_button: gtk::Button,
        pub(crate) screenshot_button: gtk::Button,
        pub(crate) new_window_button: gtk::Button,
        pub(crate) history_button: gtk::Button,
        pub(crate) settings_button: gtk::Button,
        pub(crate) about_button: gtk::Button,
        pub(crate) menu_box: gtk::Box,
        pub(crate) menu_popover: gtk::Popover,
        // Downloads popover
        pub(crate) downloads_box: gtk::Box,
        pub(crate) downloads_popover: gtk::Popover,
        // Tabs
        pub(crate) tab_bar: libadwaita::TabBar,
        pub(crate) tab_view: libadwaita::TabView,
        // Main content
        pub(crate) main_box: gtk::Box,
        pub(crate) tab_overview: libadwaita::TabOverview,
    }

    impl Window {}

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "OkuWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;
    }

    impl ObjectImpl for Window {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            Self::derived_set_property(self, id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            Self::derived_property(self, id, pspec)
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
    @extends libadwaita::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionMap, gio::ActionGroup;
}

impl Window {
    pub fn new(app: &libadwaita::Application, initial_url: Option<&str>, ipfs: &Ipfs) -> Self {
        let this: Self = glib::Object::builder::<Self>()
            .property("application", app)
            .build();
        this.set_can_focus(true);
        this.set_title(Some("Oku"));
        this.set_icon_name(Some("com.github.dirout.oku"));

        this.setup_headerbar();
        this.setup_menu_popover();
        this.setup_downloads_popover();
        this.setup_tabs();
        this.setup_main_content();
        this.setup_overview_button_clicked();
        this.setup_menu_button_clicked();
        this.setup_downloads_button_clicked();
        this.setup_tab_signals(&ipfs);

        let imp = this.imp();

        imp.initial_url
            .replace(initial_url.unwrap_or("about:blank").to_string());

        if imp.tab_view.n_pages() == 0 {
            let initial_web_view = this.new_tab_page(ipfs).0;
            initial_connect(imp.initial_url.clone().into_inner(), &initial_web_view);
        }
        this.set_content(Some(&imp.tab_overview));
        this.show();

        this
    }

    fn setup_navigation_buttons(&self) {
        let imp = self.imp();

        // Navigation bar
        imp.nav_entry.set_can_focus(true);
        imp.nav_entry.set_focusable(true);
        imp.nav_entry.set_focus_on_click(true);
        imp.nav_entry.set_editable(true);
        imp.nav_entry.set_margin_top(4);
        imp.nav_entry.set_margin_bottom(4);
        imp.nav_entry.set_hexpand(true);
        imp.nav_entry.set_truncate_multiline(true);
        imp.nav_entry
            .set_placeholder_text(Some("Enter an address … "));
        imp.nav_entry.set_input_purpose(gtk::InputPurpose::Url);
        imp.nav_entry.set_width_request(8);

        // Back button
        imp.back_button.set_can_focus(true);
        imp.back_button.set_receives_default(true);
        imp.back_button.set_halign(gtk::Align::Start);
        imp.back_button.set_margin_top(4);
        imp.back_button.set_margin_bottom(4);
        imp.back_button.set_icon_name("go-previous");
        imp.back_button.add_css_class("linked");

        // Forward button
        imp.forward_button.set_can_focus(true);
        imp.forward_button.set_receives_default(true);
        imp.forward_button.set_halign(gtk::Align::Start);
        imp.forward_button.set_margin_top(4);
        imp.forward_button.set_margin_bottom(4);
        imp.forward_button.set_icon_name("go-next");
        imp.forward_button.add_css_class("linked");

        // All navigation buttons
        imp.navigation_buttons.set_homogeneous(true);
        imp.navigation_buttons.append(&imp.back_button);
        imp.navigation_buttons.append(&imp.forward_button);
        imp.navigation_buttons.add_css_class("linked");
    }

    fn setup_left_headerbar(&self) {
        self.setup_navigation_buttons();
        let imp = self.imp();

        // Add Tab button
        imp.add_tab_button.set_can_focus(true);
        imp.add_tab_button.set_receives_default(true);
        imp.add_tab_button.set_margin_start(4);
        imp.add_tab_button.set_margin_top(4);
        imp.add_tab_button.set_margin_bottom(4);
        imp.add_tab_button.set_icon_name("list-add");

        // Refresh button
        imp.refresh_button.set_can_focus(true);
        imp.refresh_button.set_receives_default(true);
        imp.refresh_button.set_halign(gtk::Align::Start);
        imp.refresh_button.set_margin_start(4);
        imp.refresh_button.set_margin_end(8);
        imp.refresh_button.set_margin_top(4);
        imp.refresh_button.set_margin_bottom(4);
        imp.refresh_button.set_icon_name("view-refresh");

        // Left header buttons
        imp.left_header_buttons.set_margin_end(4);
        imp.left_header_buttons.append(&imp.navigation_buttons);
        imp.left_header_buttons.append(&imp.add_tab_button);
        imp.left_header_buttons.append(&imp.refresh_button);
    }

    fn setup_right_headerbar(&self) {
        let imp = self.imp();

        // Overview button
        imp.overview_button.set_can_focus(true);
        imp.overview_button.set_receives_default(true);
        imp.overview_button.set_halign(gtk::Align::Start);
        imp.overview_button.set_margin_start(4);
        imp.overview_button.set_margin_bottom(4);
        imp.overview_button.set_icon_name("view-grid-symbolic");

        // Downloads button
        imp.downloads_button.set_can_focus(true);
        imp.downloads_button.set_receives_default(true);
        imp.downloads_button.set_halign(gtk::Align::Start);
        imp.downloads_button.set_margin_start(4);
        imp.downloads_button.set_margin_bottom(4);
        imp.downloads_button
            .set_icon_name("folder-download-symbolic");

        // Find button
        imp.find_button.set_can_focus(true);
        imp.find_button.set_receives_default(true);
        imp.find_button.set_halign(gtk::Align::Start);
        imp.find_button.set_margin_start(4);
        imp.find_button.set_margin_bottom(4);
        imp.find_button.set_icon_name("edit-find");

        // IPFS menu button
        imp.ipfs_button.set_can_focus(true);
        imp.ipfs_button.set_receives_default(true);
        imp.ipfs_button.set_halign(gtk::Align::Start);
        imp.ipfs_button.set_margin_start(4);
        imp.ipfs_button.set_margin_bottom(4);
        imp.ipfs_button.set_icon_name("emblem-shared");

        // Onion routing button
        imp.tor_button.set_can_focus(true);
        imp.tor_button.set_receives_default(true);
        imp.tor_button.set_halign(gtk::Align::Start);
        imp.tor_button.set_hexpand(false);
        imp.tor_button.set_vexpand(false);
        imp.tor_button.set_overflow(gtk::Overflow::Hidden);
        imp.tor_button.set_margin_start(4);
        imp.tor_button.set_margin_bottom(4);
        imp.tor_button.set_icon_name("security-medium");

        // Menu button
        imp.menu_button.set_can_focus(true);
        imp.menu_button.set_receives_default(true);
        imp.menu_button.set_halign(gtk::Align::Start);
        imp.menu_button.set_margin_start(4);
        imp.menu_button.set_margin_bottom(4);
        imp.menu_button.set_icon_name("document-properties");

        // Right header buttons
        imp.right_header_buttons.set_margin_start(4);
        imp.right_header_buttons.set_spacing(2);
        imp.right_header_buttons.set_homogeneous(true);

        imp.right_header_buttons.append(&imp.overview_button);
        imp.right_header_buttons.append(&imp.downloads_button);
        imp.right_header_buttons.append(&imp.find_button);
        imp.right_header_buttons.append(&imp.ipfs_button);
        imp.right_header_buttons.append(&imp.tor_button);
        imp.right_header_buttons.append(&imp.menu_button);
    }

    fn setup_headerbar(&self) {
        self.setup_left_headerbar();
        self.setup_right_headerbar();
        let imp = self.imp();
        // HeaderBar
        imp.headerbar.set_can_focus(true);
        imp.headerbar.set_title_widget(Some(&imp.nav_entry));
        imp.headerbar.pack_start(&imp.left_header_buttons);
        imp.headerbar.pack_end(&imp.right_header_buttons);
    }

    fn setup_menu_popover(&self) {
        let imp = self.imp();

        // Zoom out button
        imp.zoomout_button.set_can_focus(true);
        imp.zoomout_button.set_receives_default(true);
        imp.zoomout_button.set_halign(gtk::Align::Start);
        imp.zoomout_button.set_margin_top(4);
        imp.zoomout_button.set_margin_bottom(4);
        imp.zoomout_button.set_icon_name("zoom-out");
        imp.zoomout_button.add_css_class("linked");

        // Zoom in button
        imp.zoomin_button.set_can_focus(true);
        imp.zoomin_button.set_receives_default(true);
        imp.zoomin_button.set_halign(gtk::Align::Start);
        imp.zoomin_button.set_margin_top(4);
        imp.zoomin_button.set_margin_bottom(4);
        imp.zoomin_button.set_icon_name("zoom-in");
        imp.zoomin_button.add_css_class("linked");

        // Both zoom buttons
        imp.zoom_buttons.set_homogeneous(true);
        imp.zoom_buttons.append(&imp.zoomout_button);
        imp.zoom_buttons.append(&imp.zoomin_button);
        imp.zoom_buttons.add_css_class("linked");

        // Zoom reset button
        imp.zoomreset_button.set_can_focus(true);
        imp.zoomreset_button.set_receives_default(true);
        imp.zoomreset_button.set_halign(gtk::Align::Start);
        imp.zoomreset_button.set_margin_top(4);
        imp.zoomreset_button.set_margin_bottom(4);
        imp.zoomreset_button.set_icon_name("zoom-original");

        // Fullscreen button
        imp.fullscreen_button.set_can_focus(true);
        imp.fullscreen_button.set_receives_default(true);
        imp.fullscreen_button.set_halign(gtk::Align::Start);
        imp.fullscreen_button.set_margin_top(4);
        imp.fullscreen_button.set_margin_bottom(4);
        imp.fullscreen_button
            .set_icon_name("video-display-symbolic");

        // Screenshot button
        imp.screenshot_button.set_can_focus(true);
        imp.screenshot_button.set_receives_default(true);
        imp.screenshot_button.set_halign(gtk::Align::Start);
        imp.screenshot_button.set_margin_top(4);
        imp.screenshot_button.set_margin_bottom(4);
        imp.screenshot_button.set_icon_name("camera-photo");

        // New Window button
        imp.new_window_button.set_can_focus(true);
        imp.new_window_button.set_receives_default(true);
        imp.new_window_button.set_halign(gtk::Align::Start);
        imp.new_window_button.set_margin_top(4);
        imp.new_window_button.set_margin_bottom(4);
        imp.new_window_button.set_icon_name("window-new");

        // History button
        imp.history_button.set_can_focus(true);
        imp.history_button.set_receives_default(true);
        imp.history_button.set_halign(gtk::Align::Start);
        imp.history_button.set_margin_top(4);
        imp.history_button.set_margin_bottom(4);
        imp.history_button.set_icon_name("document-open-recent");

        // Settings button
        imp.settings_button.set_can_focus(true);
        imp.settings_button.set_receives_default(true);
        imp.settings_button.set_halign(gtk::Align::Start);
        imp.settings_button.set_margin_top(4);
        imp.settings_button.set_margin_bottom(4);
        imp.settings_button.set_icon_name("preferences-system");

        // About button
        imp.about_button.set_can_focus(true);
        imp.about_button.set_receives_default(true);
        imp.about_button.set_halign(gtk::Align::Start);
        imp.about_button.set_margin_top(4);
        imp.about_button.set_margin_bottom(4);
        imp.about_button.set_icon_name("help-about");

        // Menu popover
        imp.menu_box.set_margin_start(4);
        imp.menu_box.set_margin_end(4);
        imp.menu_box.set_margin_top(4);
        imp.menu_box.set_margin_bottom(4);
        imp.menu_box.set_spacing(8);
        imp.menu_box.set_hexpand(true);
        imp.menu_box.append(&imp.zoom_buttons);
        imp.menu_box.append(&imp.zoomreset_button);
        imp.menu_box.append(&imp.fullscreen_button);
        imp.menu_box.append(&imp.screenshot_button);
        imp.menu_box.append(&imp.new_window_button);
        imp.menu_box.append(&imp.history_button);
        imp.menu_box.append(&imp.settings_button);
        imp.menu_box.append(&imp.about_button);
        imp.menu_box.add_css_class("toolbar");

        imp.menu_popover.set_child(Some(&imp.menu_box));
        imp.menu_popover.set_parent(&imp.menu_button);
    }

    fn setup_downloads_popover(&self) {
        let imp = self.imp();

        // Downloads popover
        imp.downloads_box.set_margin_start(4);
        imp.downloads_box.set_margin_end(4);
        imp.downloads_box.set_margin_top(4);
        imp.downloads_box.set_margin_bottom(4);
        imp.downloads_box.set_spacing(8);

        imp.downloads_popover.set_child(Some(&imp.downloads_box));
        imp.downloads_popover.set_parent(&imp.downloads_button);
    }

    fn setup_tabs(&self) {
        let imp = self.imp();

        imp.tab_view.set_vexpand(true);
        imp.tab_view.set_visible(true);

        imp.tab_bar.set_autohide(true);
        imp.tab_bar.set_expand_tabs(true);
        imp.tab_bar.set_view(Some(&imp.tab_view));
    }

    fn setup_main_content(&self) {
        let imp = self.imp();

        imp.main_box.set_orientation(gtk::Orientation::Vertical);
        imp.main_box.set_vexpand(true);
        imp.main_box.append(&imp.headerbar);
        imp.main_box.append(&imp.tab_bar);
        imp.main_box.append(&imp.tab_view);

        imp.tab_overview.set_enable_new_tab(true);
        imp.tab_overview.set_enable_search(true);
        imp.tab_overview.set_view(Some(&imp.tab_view));
        imp.tab_overview.set_child(Some(&imp.main_box));
    }

    fn setup_overview_button_clicked(&self) {
        let imp = self.imp();

        imp.overview_button.connect_clicked(clone!(
            #[weak(rename_to = tab_overview)]
            imp.tab_overview,
            move |_| {
                tab_overview.set_open(!tab_overview.is_open());
            }
        ));
    }

    fn setup_menu_button_clicked(&self) {
        let imp = self.imp();

        imp.menu_button.connect_clicked(clone!(
            #[weak(rename_to = menu_popover)]
            imp.menu_popover,
            move |_| {
                menu_popover.popup();
            }
        ));
    }

    fn setup_downloads_button_clicked(&self) {
        let imp = self.imp();

        imp.downloads_button.connect_clicked(clone!(
            #[weak(rename_to = downloads_popover)]
            imp.downloads_popover,
            move |_| {
                downloads_popover.popup();
            }
        ));
    }

    /// Create a new WebKit instance for the current tab
    ///
    /// # Arguments
    ///  
    /// * `verbose` - Whether browser messages should be printed onto the standard output
    ///
    /// * `is_private` - Whether the window represents a private session
    ///
    /// * `ipfs_button` - Button indicating whether the browser is using a built-in (native) IPFS handler, or an external one
    ///
    /// * `headerbar` - The browser's headerbar
    fn new_view(&self, ipfs: &Ipfs) -> webkit2gtk::WebView {
        let imp = self.imp();

        let web_settings: webkit2gtk::Settings = new_webkit_settings();
        let web_view = WebView::new();
        web_view.set_vexpand(true);
        let network_session = web_view.network_session().unwrap();
        let data_manager = network_session.website_data_manager().unwrap();
        let web_context = web_view.context().unwrap();
        let security_manager = web_context.security_manager().unwrap();
        let extensions_path = format!("{}/web-extensions/", *DATA_DIR);

        data_manager.set_favicons_enabled(true);

        web_context.register_uri_scheme(
            "ipns",
            clone!(
                #[weak]
                imp,
                #[strong]
                ipfs,
                move |request: &URISchemeRequest| {
                    let ctx = glib::MainContext::default();
                    let request_url = request.uri().unwrap().to_string();
                    let decoded_url = urlencoding::decode(&request_url).unwrap();
                    let ipns_path = format!("/ipns/{}", decoded_url.replacen("ipns://", "", 1))
                        .parse::<ipfs::IpfsPath>()
                        .unwrap();
                    let mem_stream = gio::MemoryInputStream::new();
                    ctx.spawn_local_with_priority(
                        glib::source::Priority::HIGH,
                        clone!(
                            #[weak]
                            imp,
                            #[weak]
                            request,
                            #[strong]
                            ipfs,
                            async move {
                                let resolved_ipns_path =
                                    ipfs.resolve_ipns(&ipns_path, true).await.unwrap();
                                let ipfs_stream = ipfs.cat_unixfs(resolved_ipns_path);
                                let mut file_vec: Vec<u8> = vec![];
                                pin_mut!(ipfs_stream);
                                while let Some(result) = ipfs_stream.next().await {
                                    match result {
                                        Ok(bytes) => {
                                            file_vec.extend(bytes);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error: {} (streamed {} bytes)",
                                                e,
                                                file_vec.len()
                                            );
                                            if file_vec.len() == 0 {
                                                request.finish_error(&mut glib::error::Error::new(
                                                    gio::ResolverError::NotFound,
                                                    &e.to_string(),
                                                ));
                                            }
                                            break;
                                        }
                                    }
                                }
                                let byte_size = file_vec.len();
                                mem_stream.add_bytes(&glib::Bytes::from_owned(file_vec));
                                request.finish(&mem_stream, byte_size.try_into().unwrap(), None);
                            }
                        ),
                    );
                }
            ),
        );
        web_context.register_uri_scheme(
            "ipfs",
            clone!(
                #[weak]
                imp,
                #[strong]
                ipfs,
                move |request: &URISchemeRequest| {
                    let ctx = glib::MainContext::default();
                    let request_url = request.uri().unwrap().to_string();
                    let decoded_url = urlencoding::decode(&request_url).unwrap();
                    let ipfs_path = decoded_url
                        .replacen("ipfs://", "", 1)
                        .parse::<ipfs::IpfsPath>()
                        .unwrap();
                    let mem_stream = gio::MemoryInputStream::new();
                    let ipfs_stream = ipfs.cat_unixfs(ipfs_path);
                    ctx.spawn_local_with_priority(
                        glib::source::Priority::HIGH,
                        clone!(
                            #[weak]
                            request,
                            async move {
                                let mut file_vec: Vec<u8> = vec![];
                                pin_mut!(ipfs_stream);
                                while let Some(result) = ipfs_stream.next().await {
                                    match result {
                                        Ok(bytes) => {
                                            file_vec.extend(bytes);
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Error: {} (streamed {} bytes)",
                                                e,
                                                file_vec.len()
                                            );
                                            if file_vec.len() == 0 {
                                                request.finish_error(&mut glib::error::Error::new(
                                                    gio::ResolverError::NotFound,
                                                    &e.to_string(),
                                                ));
                                            }
                                            break;
                                        }
                                    }
                                }
                                let byte_size = file_vec.len();
                                mem_stream.add_bytes(&glib::Bytes::from_owned(file_vec));
                                request.finish(&mem_stream, byte_size.try_into().unwrap(), None);
                            }
                        ),
                    );
                }
            ),
        );
        security_manager.register_uri_scheme_as_secure("ipfs");
        security_manager.register_uri_scheme_as_secure("ipns");
        web_settings.set_user_agent_with_application_details(Some("Oku"), Some(VERSION.unwrap()));
        // web_settings.set_enable_write_console_messages_to_stdout(verbose);
        web_view.set_settings(&web_settings);
        web_context.set_web_process_extensions_directory(&extensions_path);
        web_view.set_visible(true);
        web_view.set_width_request(1024);
        web_view.set_height_request(640);
        web_view.load_uri("about:blank");

        network_session.connect_download_started(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak]
            imp,
            move |_w, download| {
                libadwaita::MessageDialog::new(
                    gtk::Window::NONE,
                    Some("Download file?"),
                    Some(&format!(
                        "Would you like to download '{}'?",
                        download.request().unwrap().uri().unwrap()
                    )),
                );
            }
        ));

        web_view.connect_title_notify(clone!(
            #[weak(rename_to = tab_view)]
            imp.tab_view,
            #[weak]
            imp,
            move |w| update_title(tab_view, &w)
        ));
        web_view.connect_favicon_notify(clone!(
            #[weak(rename_to = tab_view)]
            imp.tab_view,
            #[weak]
            imp,
            move |w| update_favicon(tab_view, &w)
        ));
        web_view.connect_load_changed(clone!(
            #[weak(rename_to = tab_view)]
            imp.tab_view,
            #[weak]
            imp,
            move |w, _| {
                imp.obj().set_title(Some(
                    &w.title()
                        .unwrap_or_else(|| glib::GString::from("Oku"))
                        .to_string(),
                ));
                update_favicon(tab_view, &w);
            }
        ));
        web_view.connect_enter_fullscreen(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak]
            imp,
            #[upgrade_or]
            false,
            move |w| {
                imp.headerbar.set_visible(false);
                imp.tab_bar.set_visible(false);
                imp.obj().set_fullscreened(true);
                imp.tab_bar.hide();
                imp.tab_bar.set_opacity(0.0);
                true
            }
        ));
        web_view.connect_leave_fullscreen(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak]
            imp,
            #[weak]
            web_view,
            #[upgrade_or]
            false,
            move |w| {
                imp.headerbar.set_visible(true);
                imp.tab_bar.set_visible(true);
                imp.obj().set_fullscreened(false);
                imp.tab_bar.show();
                imp.tab_bar.set_opacity(100.0);
                true
            }
        ));

        web_view
    }

    /// Create a new entry in the TabBar
    ///
    /// # Arguments
    ///
    /// * `tabs` - The TabBar containing the tabs of the current browser session
    ///  
    /// * `verbose` - Whether browser messages should be printed onto the standard output
    ///
    /// * `is_private` - Whether the window represents a private session
    ///
    /// * `ipfs_button` - Button indicating whether the browser is using a built-in (native) IPFS handler, or an external one
    ///
    /// * `headerbar` - The browser's headerbar
    pub fn new_tab_page(&self, ipfs: &Ipfs) -> (webkit2gtk::WebView, libadwaita::TabPage) {
        let imp = self.imp();

        let tab_view = imp.tab_bar.view().unwrap();
        let new_view = self.new_view(&ipfs);
        let new_page = tab_view.append(&new_view);
        new_page.set_title("New Tab");
        new_page.set_icon(Some(&gio::ThemedIcon::new("content-loading-symbolic")));
        new_page.set_live_thumbnail(true);
        tab_view.set_selected_page(&new_page);
        new_page.set_indicator_icon(Some(&gio::ThemedIcon::new("view-pin-symbolic")));
        new_page.set_indicator_activatable(true);
        // Indicator appearance
        new_view.connect_is_muted_notify(clone!(
            #[weak]
            new_view,
            #[weak]
            new_page,
            #[weak]
            tab_view,
            #[weak]
            imp,
            move |the_view| {
                // Has been muted
                if the_view.is_muted() {
                    new_page.set_indicator_icon(Some(&gio::ThemedIcon::new("audio-volume-muted")));
                    new_page.set_indicator_activatable(true);
                } else {
                    // Has been unmuted, audio is playing
                    if the_view.is_playing_audio() {
                        new_page
                            .set_indicator_icon(Some(&gio::ThemedIcon::new("audio-volume-high")));
                        new_page.set_indicator_activatable(true);
                    }
                    // Has been unmuted, audio is not playing
                    else {
                        new_page
                            .set_indicator_icon(Some(&gio::ThemedIcon::new("view-pin-symbolic")));
                        new_page.set_indicator_activatable(true);
                    }
                }
            }
        ));
        new_view.connect_is_playing_audio_notify(clone!(
            #[weak]
            new_view,
            #[weak]
            new_page,
            #[weak]
            tab_view,
            #[weak]
            imp,
            move |the_view| {
                // Audio has started playing and not muted
                if the_view.is_playing_audio() && !the_view.is_muted() {
                    new_page.set_indicator_icon(Some(&gio::ThemedIcon::new("audio-volume-high")));
                    new_page.set_indicator_activatable(true);
                } else if !the_view.is_playing_audio() {
                    // Audio has stopped playing, muted
                    if the_view.is_muted() {
                        new_page
                            .set_indicator_icon(Some(&gio::ThemedIcon::new("audio-volume-muted")));
                        new_page.set_indicator_activatable(true);
                    } else {
                        // Audio has stopped playing, not muted
                        new_page
                            .set_indicator_icon(Some(&gio::ThemedIcon::new("view-pin-symbolic")));
                        new_page.set_indicator_activatable(true);
                    }
                }
            }
        ));
        new_view.connect_uri_notify(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak(rename_to = headerbar)]
            imp.headerbar,
            #[weak(rename_to = nav_entry)]
            imp.nav_entry,
            #[weak]
            new_view,
            #[weak]
            imp,
            move |w| { update_nav_bar(&nav_entry, &w) }
        ));
        new_view.connect_estimated_load_progress_notify(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak(rename_to = headerbar)]
            imp.headerbar,
            #[weak(rename_to = nav_entry)]
            imp.nav_entry,
            #[weak]
            new_view,
            #[weak]
            imp,
            move |w| {
                let tab_view: libadwaita::TabView = tab_bar.view().unwrap();
                let current_page = tab_view.page(w);
                current_page.set_loading(true);
                update_load_progress(&nav_entry, &w)
            }
        ));
        new_view.connect_is_loading_notify(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak]
            imp,
            move |w| {
                let tab_view: libadwaita::TabView = tab_bar.view().unwrap();
                let current_page = tab_view.page(w);
                current_page.set_loading(w.is_loading())
            }
        ));
        (new_view, new_page)
    }

    pub fn setup_tab_signals(&self, ipfs: &Ipfs) {
        let imp = self.imp();

        imp.tab_overview.connect_create_tab(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak(rename_to = ipfs_button)]
            imp.ipfs_button,
            #[weak(rename_to = headerbar)]
            imp.headerbar,
            #[weak]
            imp,
            #[weak(rename_to = this)]
            self,
            #[strong]
            ipfs,
            #[upgrade_or_panic]
            move |_| { this.new_tab_page(&ipfs).1 }
        ));

        // Add Tab button clicked
        imp.add_tab_button.connect_clicked(clone!(
            #[weak(rename_to = tab_bar)]
            imp.tab_bar,
            #[weak(rename_to = ipfs_button)]
            imp.ipfs_button,
            #[weak(rename_to = headerbar)]
            imp.headerbar,
            #[weak]
            imp,
            #[weak(rename_to = this)]
            self,
            #[strong]
            ipfs,
            move |_| {
                this.new_tab_page(&ipfs);
            }
        ));
    }
}