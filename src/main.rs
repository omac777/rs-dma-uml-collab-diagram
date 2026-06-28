// Complete implementation with all features
// This is a large file - implementing: selection, editing, association workflow, directory dialogs, type parsing

slint::include_modules!();

use rs_dma_lib::{RucdProject, ObjectInstantiation, Association, Message, RustType, rust_parser::ingest_rust_sources, io::{save_project, load_project}};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("rust-side: Rust UML Collaboration Diagram Editor v0.2");
    println!("rust-side: Features: Selection | Editing | Associations | Directory Import");
    
    let main_window = MainWindow::new().expect("Failed to create window");
    
    // Application state
    let project = Rc::new(RefCell::new(RucdProject::new("Untitled")));
    let selected_obj_id = Rc::new(RefCell::new(Option::<String>::None));
    let assoc_source_id = Rc::new(RefCell::new(None::<String>));
    let ingested_types = Rc::new(RefCell::new(Vec::<RustType>::new()));
    let next_msg_seq = Rc::new(RefCell::new(1));
    
    render_all(&main_window, &*project.borrow(), selected_obj_id.borrow().as_deref());

    // === NEW PROJECT ===
    let proj1 = project.clone();
    let sel1 = selected_obj_id.clone();
    let win1 = main_window.clone_strong();
    main_window.on_new_project(move || {
        *proj1.borrow_mut() = RucdProject::new("Untitled");
        *sel1.borrow_mut() = None;
        win1.set_project_name("Untitled".into());
        render_all(&win1, &*proj1.borrow(), None);
        println!("rust-side: ✓ New project created");
    });

    // === OPEN PROJECT ===
    let proj2 = project.clone();
    let sel2 = selected_obj_id.clone();
    let win2 = main_window.clone_strong();
    main_window.on_open_project(move || {
        let dialog = rfd::FileDialog::new()
            .add_filter("RUCD", &["rucd", "json"])
            .pick_file();
        
        if let Some(path) = dialog {
            match load_project(&path) {
                Ok(p) => {
                    *proj2.borrow_mut() = p;
                    *sel2.borrow_mut() = None;
                    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Project");
                    win2.set_project_name(filename.into());
                    render_all(&win2, &*proj2.borrow(), None);
                    println!("rust-side: ✓ Opened: {}", path.display());
                }
                Err(e) => println!("rust-side: ✗ Load failed: {}", e),
            }
        }
    });

    // === SAVE PROJECT ===
    let proj3 = project.clone();
    main_window.on_save_project(move || {
        let dialog = rfd::FileDialog::new()
            .add_filter("RUCD", &["rucd"])
            .set_file_name("diagram.rucd")
            .pick_file();
        
        if let Some(path) = dialog {
            let path = if path.extension().is_none() {
                let mut p = path.clone();
                p.set_extension("rucd");
                p
            } else {
                path
            };
            
            match save_project(&*proj3.borrow(), &path) {
                Ok(_) => println!("rust-side: ✓ Saved: {}", path.display()),
                Err(e) => println!("rust-side: ✗ Save failed: {}", e),
            }
        }
    });

    // === INGEST RUST DIRECTORY ===
    let proj4 = project.clone();
    let types4 = ingested_types.clone();
    let sel4 = selected_obj_id.clone();
    let win4 = main_window.clone_strong();
    main_window.on_ingest_sources(move || {
        let dialog = rfd::FileDialog::new()
            .pick_folder();
        
        if let Some(path) = dialog {
            println!("rust-side: Scanning directory: {}", path.display());
            let types = ingest_rust_sources(&path).unwrap_or_default();
            println!("rust-side: ✓ Found {} types/traits", types.len());
            
            *types4.borrow_mut() = types;
            *sel4.borrow_mut() = None;
            
            // Update UI with first few type names
            let types_borrow = types4.borrow();
            let preview: String = types_borrow.iter().take(5)
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            
            win4.set_ingested_types_count(types_borrow.len() as i32);
            println!("rust-side: Types available: {}{}", preview, if types_borrow.len() > 5 { "..." } else { "" });
        }
    });

    // === ADD OBJECT (from ingested types with dialog) ===
    let proj5 = project.clone();
    let types5a = ingested_types.clone();
    let types5b = ingested_types.clone();
    let sel5 = selected_obj_id.clone();
    let win5a1 = main_window.clone_strong();
    let win5a2 = main_window.clone_strong();
    let win5a3 = main_window.clone_strong();
    let win5a4a = main_window.clone_strong();
    let win5a4b = main_window.clone_strong();
    let win5a5 = main_window.clone_strong();
    let win5b = main_window.clone_strong();
    let proj5b = project.clone();
    
    main_window.on_add_object(move || {
        let types_borrow = types5a.borrow();
        if types_borrow.is_empty() {
            println!("rust-side: ⚠ No types ingested. Click 'Ingest Rust' first.");
            return;
        }
        
        // Populate type list (up to 4 visible in dialog)
        let type1 = types_borrow.get(0).map(|t| t.name.as_str()).unwrap_or("");
        let type2 = types_borrow.get(1).map(|t| t.name.as_str()).unwrap_or("");
        let type3 = types_borrow.get(2).map(|t| t.name.as_str()).unwrap_or("");
        let type4 = types_borrow.get(3).map(|t| t.name.as_str()).unwrap_or("");
        
        win5a1.set_dialog_selected_type(type1.into());
        win5a1.set_dialog_object_name("".into());
        win5a1.set_ingested_types_count(types_borrow.len() as i32);
        win5a1.set_type_selector_type1(type1.into());
        win5a1.set_type_selector_type2(type2.into());
        win5a1.set_type_selector_type3(type3.into());
        win5a1.set_type_selector_type4(type4.into());
        win5a1.set_show_type_dialog(true);
        println!("rust-side: → Add Object dialog opened ({} types)", types_borrow.len());
    });
    
    // Type selection callbacks
    let types_for_t1 = ingested_types.clone();
    let types_for_t2 = ingested_types.clone();
    let types_for_t3 = ingested_types.clone();
    let types_for_t4 = ingested_types.clone();
    
    main_window.on_type1_clicked(move || {
        let types_borrow = types_for_t1.borrow();
        if let Some(t) = types_borrow.get(0) {
            win5a5.set_dialog_selected_type(t.name.clone().into());
        }
    });
    main_window.on_type2_clicked(move || {
        let types_borrow = types_for_t2.borrow();
        if let Some(t) = types_borrow.get(1) {
            win5a3.set_dialog_selected_type(t.name.clone().into());
        }
    });
    main_window.on_type3_clicked(move || {
        let types_borrow = types_for_t3.borrow();
        if let Some(t) = types_borrow.get(2) {
            win5a4a.set_dialog_selected_type(t.name.clone().into());
        }
    });
    main_window.on_type4_clicked(move || {
        let types_borrow = types_for_t4.borrow();
        if let Some(t) = types_borrow.get(3) {
            win5a4b.set_dialog_selected_type(t.name.clone().into());
        }
    });
    
    // Dialog confirmed - create the object
    main_window.on_dialog_confirmed(move || {
        let type_name = win5b.get_dialog_selected_type().to_string();
        let obj_name = win5b.get_dialog_object_name().to_string();
        let types_borrow = types5b.borrow();
        
        if type_name.is_empty() {
            println!("rust-side: ⚠ No type selected");
            win5b.set_show_type_dialog(false);
            return;
        }
        
        // Find the type (use the selected type name)
        let rust_type = types_borrow.iter()
            .find(|t| t.name == type_name);
        
        if rust_type.is_none() {
            println!("rust-side: ⚠ Type not found: {}", type_name);
            win5b.set_show_type_dialog(false);
            return;
        }
        
        let rust_type = rust_type.unwrap();
        
        // Use user-provided name or auto-generate
        let display_name = if obj_name.trim().is_empty() {
            format!("{}: {}", rust_type.name.to_lowercase(), rust_type.name)
        } else {
            format!("{}: {}", obj_name.trim(), rust_type.name)
        };
        
        let mut proj_mut = proj5b.borrow_mut();
        let obj_num = proj_mut.objects.len() + 1;
        let row = ((obj_num - 1) / 4) as f64;
        let col = ((obj_num - 1) % 4) as f64;
        
        let obj = ObjectInstantiation {
            id: format!("obj{}", obj_num),
            name: display_name,
            type_name: rust_type.name.clone(),
            is_trait: rust_type.is_trait,
            x: 100.0 + (col * 200.0),
            y: 100.0 + (row * 150.0),
            width: 160.0,
            height: 90.0,
        };
        
        println!("rust-side: ✓ Added object: {}", obj.name);
        proj_mut.objects.push(obj);
        drop(proj_mut);
        
        win5b.set_show_type_dialog(false);
        
        // Auto-layout after adding
        layout_objects_internal(&proj5);
        render_all(&win5b, &*proj5b.borrow(), sel5.borrow().as_deref());
    });
    
    // Dialog cancelled
    main_window.on_dialog_cancelled(move || {
        win5a2.set_show_type_dialog(false);
        println!("rust-side: ✗ Add Object cancelled");
    });

    // === ASSOCIATION CREATION (click 2 objects) ===
    let proj6 = project.clone();
    let assoc_src = assoc_source_id.clone();
    let sel6 = selected_obj_id.clone();
    let win6a = main_window.clone_strong();
    main_window.on_start_association(move || {
        *assoc_src.borrow_mut() = None;
        win6a.set_creating_association(true);
        println!("rust-side: → Association mode: Click first object, then second object");
    });

    let win6b = main_window.clone_strong();
    let proj6b = project.clone();
    let assoc_src2 = assoc_source_id.clone();
    let sel6b = selected_obj_id.clone();
    main_window.on_object_clicked(move |obj_id: slint::SharedString| {
        let id = obj_id.to_string();
        let is_assoc_mode = win6b.get_creating_association();
        
        if is_assoc_mode {
            let src = assoc_src2.borrow().clone();
            if src.is_none() {
                // First object clicked
                *assoc_src2.borrow_mut() = Some(id.clone());
                *sel6b.borrow_mut() = Some(id.clone());
                println!("rust-side: → Selected first object: {}", id);
                
                // Highlight the selected object
                render_all(&win6b, &*proj6b.borrow(), Some(&id));
            } else {
                // Second object clicked - create association
                let target = id;
                let source = src.unwrap();
                
                if source == target {
                    println!("rust-side:  Can't associate object with itself");
                    win6b.set_creating_association(false);
                    *sel6b.borrow_mut() = None;
                    render_all(&win6b, &*proj6b.borrow(), None);
                    return;
                }
                
                let mut proj_mut = proj6b.borrow_mut();
                
                // Check if association already exists
                let exists = proj_mut.associations.iter()
                    .any(|a| (a.source_id == source && a.target_id == target) ||
                             (a.source_id == target && a.target_id == source));
                
                if exists {
                    println!("rust-side: ⚠ Association already exists between {} and {}", source, target);
                    win6b.set_creating_association(false);
                    *sel6b.borrow_mut() = None;
                    render_all(&win6b, &*proj6b.borrow(), None);
                    return;
                }
                
                proj_mut.associations.push(Association {
                    id: format!("assoc_{}_{}", source, target),
                    source_id: source.clone(),
                    target_id: target.clone(),
                });
                
                println!("rust-side: ✓ Created association: {} → {}", source, target);
                win6b.set_creating_association(false);
                *assoc_src2.borrow_mut() = None;
                *sel6b.borrow_mut() = None;
                drop(proj_mut);
                
                render_all(&win6b, &*proj6b.borrow(), None);
            }
        } else {
            // Normal selection mode
            *sel6b.borrow_mut() = Some(id.clone());
            println!("rust-side: ✓ Selected object: {}", id);
            render_all(&win6b, &*proj6b.borrow(), Some(&id));
        }
    });

    // === MESSAGE CREATION (Phase 4b) ===
    let msg_proj = project.clone();
    let msg_window = main_window.clone_strong();
    let msg_ingested = ingested_types.clone();
    
    main_window.on_add_message(move || {
        msg_window.set_creating_message(true);
        
        // Clone data we need before mutable borrow
        let (assoc_id, target_id, source_id) = {
            let proj_borrow = msg_proj.borrow();
            if proj_borrow.associations.is_empty() {
                println!("rust-side: ⚠ No associations");
                drop(proj_borrow);
                msg_window.set_creating_message(false);
                return;
            }
            let assoc = &proj_borrow.associations[0];
            (assoc.id.clone(), assoc.target_id.clone(), assoc.source_id.clone())
        };
        
        // Get service name
        let service_name = {
            let proj_borrow = msg_proj.borrow();
            if let Some(obj) = proj_borrow.objects.iter().find(|o| o.id == target_id) {
                let types = msg_ingested.borrow();
                types.iter().find(|t| t.name == obj.type_name)
                    .and_then(|t| t.methods.first().cloned())
                    .unwrap_or_else(|| "process_request".to_string())
            } else {
                "service_call".to_string()
            }
        };
        
        // Create message
        {
            let mut proj_mut = msg_proj.borrow_mut();
            let msg_count = proj_mut.messages.iter()
                .filter(|m| m.association_id == assoc_id).count();
            let seq_num = format!("{}.1", msg_count + 1);
            let msg_id = format!("msg_{}", proj_mut.messages.len());
            
            proj_mut.messages.push(Message {
                id: msg_id,
                association_id: assoc_id,
                sequence_number: seq_num.clone(),
                service_name: service_name.clone(),
                source_object_id: source_id,
                target_object_id: target_id,
            });
            
            println!("rust-side: ✓ Message created: {} → {}", seq_num, service_name);
        }
        
        msg_window.set_creating_message(false);
        render_all(&msg_window, &*msg_proj.borrow(), None);
    });

    // Association line clicked (when in message mode)  
    let msg_proj2 = project.clone();
    let msg_window2 = main_window.clone_strong();
    let msg_ingested2 = ingested_types.clone();
    
    main_window.on_association_line_clicked(move |_assoc_id: slint::SharedString| {
        let is_msg_mode = msg_window2.get_creating_message();
        
        if !is_msg_mode { return; }
        
        // Get first association for demo
        let (assoc_id, target_id, source_id) = {
            let proj_borrow = msg_proj2.borrow();
            if proj_borrow.associations.is_empty() {
                println!("rust-side: ⚠ No associations");
                return;
            }
            let assoc = &proj_borrow.associations[0];
            (assoc.id.clone(), assoc.target_id.clone(), assoc.source_id.clone())
        };
        
        println!("rust-side: → Association clicked for message: {}", assoc_id);
        
        // Get service name
        let service_name = {
            let proj_borrow = msg_proj2.borrow();
            if let Some(obj) = proj_borrow.objects.iter().find(|o| o.id == target_id) {
                let types = msg_ingested2.borrow();
                types.iter().find(|t| t.name == obj.type_name)
                    .and_then(|t| t.methods.first().cloned())
                    .unwrap_or_else(|| "process_request".to_string())
            } else {
                "service_call".to_string()
            }
        };
        
        // Create message
        {
            let mut proj_mut = msg_proj2.borrow_mut();
            let msg_count = proj_mut.messages.iter()
                .filter(|m| m.association_id == assoc_id).count();
            let seq_num = format!("{}.1", msg_count + 1);
            let msg_id = format!("msg_{}", proj_mut.messages.len());
            
            proj_mut.messages.push(Message {
                id: msg_id,
                association_id: assoc_id,
                sequence_number: seq_num.clone(),
                service_name: service_name.clone(),
                source_object_id: source_id,
                target_object_id: target_id,
            });
            
            println!("rust-side: ✓ Message created: {} → {}", seq_num, service_name);
        }
        
        msg_window2.set_creating_message(false);
        render_all(&msg_window2, &*msg_proj2.borrow(), None);
    });

    // === SHOW SAMPLE OBJECTS ===
    let win7 = main_window.clone_strong();
    let proj7 = project.clone();
    let sel7b = selected_obj_id.clone();
    main_window.on_show_sample(move || {
        let mut pb = proj7.borrow_mut();
        pb.objects.clear();
        pb.associations.clear();
        pb.messages.clear();
        
        // Create 3 objects in triangle layout
        pb.objects.push(ObjectInstantiation {
            id: "A".into(),
            name: "A: App".into(),
            type_name: "App".into(),
            is_trait: false,
            x: 300.0,
            y: 100.0,
            width: 150.0,
            height: 80.0,
        });
        pb.objects.push(ObjectInstantiation {
            id: "B".into(),
            name: "B: Database".into(),
            type_name: "Database".into(),
            is_trait: false,
            x: 100.0,
            y: 400.0,
            width: 150.0,
            height: 80.0,
        });
        pb.objects.push(ObjectInstantiation {
            id: "C".into(),
            name: "C: Cache".into(),
            type_name: "Cache".into(),
            is_trait: false,
            x: 500.0,
            y: 400.0,
            width: 150.0,
            height: 80.0,
        });
        
        // Create 3 associations: (A,B), (B,C), (C,A)
        pb.associations.push(Association { id: "AB".into(), source_id: "A".into(), target_id: "B".into() });
        pb.associations.push(Association { id: "BC".into(), source_id: "B".into(), target_id: "C".into() });
        pb.associations.push(Association { id: "CA".into(), source_id: "C".into(), target_id: "A".into() });
        
        // Create 9 messages: 3 per association
        pb.messages.push(Message { id: "msg1".into(), association_id: "AB".into(), sequence_number: "1.1".into(), service_name: "message1".into(), source_object_id: "A".into(), target_object_id: "B".into() });
        pb.messages.push(Message { id: "msg2".into(), association_id: "AB".into(), sequence_number: "2.1".into(), service_name: "message2".into(), source_object_id: "A".into(), target_object_id: "B".into() });
        pb.messages.push(Message { id: "msg3".into(), association_id: "AB".into(), sequence_number: "3.1".into(), service_name: "message3".into(), source_object_id: "A".into(), target_object_id: "B".into() });
        
        pb.messages.push(Message { id: "msg4".into(), association_id: "BC".into(), sequence_number: "1.2".into(), service_name: "message4".into(), source_object_id: "B".into(), target_object_id: "C".into() });
        pb.messages.push(Message { id: "msg5".into(), association_id: "BC".into(), sequence_number: "2.2".into(), service_name: "message5".into(), source_object_id: "B".into(), target_object_id: "C".into() });
        pb.messages.push(Message { id: "msg6".into(), association_id: "BC".into(), sequence_number: "3.2".into(), service_name: "message6".into(), source_object_id: "B".into(), target_object_id: "C".into() });
        
        pb.messages.push(Message { id: "msg7".into(), association_id: "CA".into(), sequence_number: "1.3".into(), service_name: "message7".into(), source_object_id: "C".into(), target_object_id: "A".into() });
        pb.messages.push(Message { id: "msg8".into(), association_id: "CA".into(), sequence_number: "2.3".into(), service_name: "message8".into(), source_object_id: "C".into(), target_object_id: "A".into() });
        pb.messages.push(Message { id: "msg9".into(), association_id: "CA".into(), sequence_number: "3.3".into(), service_name: "message9".into(), source_object_id: "C".into(), target_object_id: "A".into() });
        
        drop(pb);
        *sel7b.borrow_mut() = None;
        render_all(&win7, &*proj7.borrow(), None);
        println!("rust-side: ✓ Sample objects loaded (triangle A-B-C with 9 messages)");
    });

    // === LAYOUT OBJECTS ===
    let win8 = main_window.clone_strong();
    let assoc_src3 = assoc_source_id.clone();
    let sel8 = selected_obj_id.clone();
    main_window.on_layout_objects(move || {
        layout_objects_internal(&project);
        win8.set_creating_association(false);
        *assoc_src3.borrow_mut() = None;
        render_all(&win8, &*project.borrow(), sel8.borrow().as_deref());
    });

    println!("rust-side: Ready! Features:");
    println!("rust-side:   • Ingest Rust: Scan directory for types");
    println!("rust-side:   • +Object: Add objects with type selection dialog");
    println!("rust-side:   • Association: Click mode, then select source→destination");
    println!("rust-side:   • Message: Click mode, then select association (TODO)");
    println!("rust-side:   • Sample Objects: Demo triangle with 9 messages");
    
    main_window.run().expect("Event loop error");
}

fn render_all(window: &MainWindow, proj: &RucdProject, selected_id: Option<&str>) {
    // Clear all first
    clear_ui(window);
    
    // Render objects
    for (i, obj) in proj.objects.iter().take(10).enumerate() {
        let is_selected = Some(obj.id.as_str()) == selected_id;
        let top_left_x = obj.x;
        let top_left_y = obj.y;

        match i {
            0 => { window.set_obj1_id(obj.id.clone().into()); window.set_obj1_name(obj.name.clone().into()); window.set_obj1_type(obj.type_name.clone().into()); window.set_obj1_center_x(top_left_x as f32); window.set_obj1_center_y(top_left_y as f32); window.set_obj1_visible(true); window.set_obj1_selected(is_selected); }
            1 => { window.set_obj2_id(obj.id.clone().into()); window.set_obj2_name(obj.name.clone().into()); window.set_obj2_type(obj.type_name.clone().into()); window.set_obj2_center_x(top_left_x as f32); window.set_obj2_center_y(top_left_y as f32); window.set_obj2_visible(true); window.set_obj2_selected(is_selected); }
            2 => { window.set_obj3_id(obj.id.clone().into()); window.set_obj3_name(obj.name.clone().into()); window.set_obj3_type(obj.type_name.clone().into()); window.set_obj3_center_x(top_left_x as f32); window.set_obj3_center_y(top_left_y as f32); window.set_obj3_visible(true); window.set_obj3_selected(is_selected); }
            3 => { window.set_obj4_id(obj.id.clone().into()); window.set_obj4_name(obj.name.clone().into()); window.set_obj4_type(obj.type_name.clone().into()); window.set_obj4_center_x(top_left_x as f32); window.set_obj4_center_y(top_left_y as f32); window.set_obj4_visible(true); window.set_obj4_selected(is_selected); }
            4 => { window.set_obj5_id(obj.id.clone().into()); window.set_obj5_name(obj.name.clone().into()); window.set_obj5_type(obj.type_name.clone().into()); window.set_obj5_center_x(top_left_x as f32); window.set_obj5_center_y(top_left_y as f32); window.set_obj5_visible(true); window.set_obj5_selected(is_selected); }
            5 => { window.set_obj6_id(obj.id.clone().into()); window.set_obj6_name(obj.name.clone().into()); window.set_obj6_type(obj.type_name.clone().into()); window.set_obj6_center_x(top_left_x as f32); window.set_obj6_center_y(top_left_y as f32); window.set_obj6_visible(true); window.set_obj6_selected(is_selected); }
            6 => { window.set_obj7_id(obj.id.clone().into()); window.set_obj7_name(obj.name.clone().into()); window.set_obj7_type(obj.type_name.clone().into()); window.set_obj7_center_x(top_left_x as f32); window.set_obj7_center_y(top_left_y as f32); window.set_obj7_visible(true); window.set_obj7_selected(is_selected); }
            7 => { window.set_obj8_id(obj.id.clone().into()); window.set_obj8_name(obj.name.clone().into()); window.set_obj8_type(obj.type_name.clone().into()); window.set_obj8_center_x(top_left_x as f32); window.set_obj8_center_y(top_left_y as f32); window.set_obj8_visible(true); window.set_obj8_selected(is_selected); }
            8 => { window.set_obj9_id(obj.id.clone().into()); window.set_obj9_name(obj.name.clone().into()); window.set_obj9_center_x(top_left_x as f32); window.set_obj9_center_y(top_left_y as f32); window.set_obj9_visible(true); window.set_obj9_selected(is_selected); }
            9 => { window.set_obj10_id(obj.id.clone().into()); window.set_obj10_name(obj.name.clone().into()); window.set_obj10_type(obj.type_name.clone().into()); window.set_obj10_center_x(top_left_x as f32); window.set_obj10_center_y(top_left_y as f32); window.set_obj10_visible(true); window.set_obj10_selected(is_selected); }
            _ => {}
        }
    }

    // Render associations (drawn BEFORE objects so they appear behind)
    for (i, assoc) in proj.associations.iter().take(5).enumerate() {
	if let Some((src, tgt)) = find_association_endpoints(proj, assoc) {
            let dx = tgt.0 - src.0;
            let dy = tgt.1 - src.1;
            let length = ((dx * dx) + (dy * dy)).sqrt();
            let angle = dy.atan2(dx);
            let base_angle_deg = angle.to_degrees() as f32;

            let mid_x = (src.0 + tgt.0) / 2.0;
            let mid_y = (src.1 + tgt.1) / 2.0;

            let mut top_label_x = mid_x;
            let mut top_label_y = mid_y;
            if length > 0.0 {
		let msgs_on_assoc: Vec<&Message> = proj.messages.iter()
                    .filter(|m| m.association_id == assoc.id)
                    .collect();
		let n = msgs_on_assoc.len();
		
		let perp_x = -dy / length;
		let perp_y = dx / length;
		
		let spacing = 18.0;
		let msg_stack_half_extent = if n > 0 { ((n as f64 - 1.0) / 2.0) * spacing } else { 0.0 };
		let yellow_offset_mag = msg_stack_half_extent + spacing;
		
		// Always place yellow box on the same side as the first message (index 0)
		// The first message is at: mid + perp * (0 - (n-1)/2) * spacing
		// which simplifies to: mid - perp * positive_value
		// So we place the yellow box at: mid - perp * yellow_offset_mag
		top_label_x = mid_x - perp_x * yellow_offset_mag;
		top_label_y = mid_y - perp_y * yellow_offset_mag;
            }

            match i {
		0 => { window.set_assoc1_x(mid_x as f32); window.set_assoc1_y(mid_y as f32); window.set_assoc1_len(length as f32); window.set_assoc1_angle(base_angle_deg as f32); window.set_assoc1_visible(true); window.set_assoc1_top_label_x(top_label_x as f32); window.set_assoc1_top_label_y(top_label_y as f32); }
		1 => { window.set_assoc2_x(mid_x as f32); window.set_assoc2_y(mid_y as f32); window.set_assoc2_len(length as f32); window.set_assoc2_angle(base_angle_deg as f32); window.set_assoc2_visible(true); window.set_assoc2_top_label_x(top_label_x as f32); window.set_assoc2_top_label_y(top_label_y as f32); }
		2 => { window.set_assoc3_x(mid_x as f32); window.set_assoc3_y(mid_y as f32); window.set_assoc3_len(length as f32); window.set_assoc3_angle(base_angle_deg as f32); window.set_assoc3_visible(true); window.set_assoc3_top_label_x(top_label_x as f32); window.set_assoc3_top_label_y(top_label_y as f32); }
		3 => { window.set_assoc4_x(mid_x as f32); window.set_assoc4_y(mid_y as f32); window.set_assoc4_len(length as f32); window.set_assoc4_angle(base_angle_deg as f32); window.set_assoc4_visible(true); window.set_assoc4_top_label_x(top_label_x as f32); window.set_assoc4_top_label_y(top_label_y as f32); }
		4 => { window.set_assoc5_x(mid_x as f32); window.set_assoc5_y(mid_y as f32); window.set_assoc5_len(length as f32); window.set_assoc5_angle(base_angle_deg as f32); window.set_assoc5_visible(true); window.set_assoc5_top_label_x(top_label_x as f32); window.set_assoc5_top_label_y(top_label_y as f32); }
		_ => {}
            }
	}
    }
    
    // Render messages (positioned along their association lines)
    for (i, msg) in proj.messages.iter().take(9).enumerate() {
        if let Some(assoc) = proj.associations.iter().find(|a| a.id == msg.association_id) {
            if let Some((src, tgt)) = find_association_endpoints(proj, assoc) {
                let dx = tgt.0 - src.0;
                let dy = tgt.1 - src.1;
                let length = ((dx * dx) + (dy * dy)).sqrt();
                
                let is_reversed = msg.source_object_id == assoc.target_id;
                
                let base_angle = dy.atan2(dx);
                let final_angle = if is_reversed { base_angle + std::f64::consts::PI } else { base_angle };
                let angle_deg = final_angle.to_degrees() as f32;
                
                let msgs_on_assoc: Vec<_> = proj.messages.iter()
                    .filter(|m| m.association_id == msg.association_id)
                    .collect();
                let msg_count = msgs_on_assoc.len() as f64;
                let msg_index = msgs_on_assoc.iter()
                    .position(|m| m.id == msg.id)
                    .unwrap_or(0) as f64;
                
                let mid_x = (src.0 + tgt.0) / 2.0;
                let mid_y = (src.1 + tgt.1) / 2.0;
                
                let spacing = 18.0;
                let center_offset = (msg_index - (msg_count - 1.0) / 2.0) * spacing;
                let perp_x = -dy / length * center_offset;
                let perp_y = dx / length * center_offset;
                
                let msg_x = mid_x + perp_x;
                let msg_y = mid_y + perp_y;
                
                let dir_str = if is_reversed { "<-" } else { "->" };
                println!("rust-side: MSG[{}] id={} assoc={} {} {} {} angle={:.1}° pos=({:.0},{:.0}) vis=true dir={}",
                    i, msg.id, msg.association_id,
                    msg.source_object_id, dir_str, msg.target_object_id,
                    angle_deg, msg_x, msg_y, dir_str);
                
                match i {
                    0 => { window.set_msg1_seq(msg.sequence_number.clone().into()); window.set_msg1_svc(msg.service_name.clone().into()); window.set_msg1_x(msg_x as f32); window.set_msg1_y(msg_y as f32); window.set_msg1_angle(angle_deg as f32); window.set_msg1_vis(true); }
                    1 => { window.set_msg2_seq(msg.sequence_number.clone().into()); window.set_msg2_svc(msg.service_name.clone().into()); window.set_msg2_x(msg_x as f32); window.set_msg2_y(msg_y as f32); window.set_msg2_angle(angle_deg as f32); window.set_msg2_vis(true); }
                    2 => { window.set_msg3_seq(msg.sequence_number.clone().into()); window.set_msg3_svc(msg.service_name.clone().into()); window.set_msg3_x(msg_x as f32); window.set_msg3_y(msg_y as f32); window.set_msg3_angle(angle_deg as f32); window.set_msg3_vis(true); }
                    3 => { window.set_msg4_seq(msg.sequence_number.clone().into()); window.set_msg4_svc(msg.service_name.clone().into()); window.set_msg4_x(msg_x as f32); window.set_msg4_y(msg_y as f32); window.set_msg4_angle(angle_deg as f32); window.set_msg4_vis(true); }
                    4 => { window.set_msg5_seq(msg.sequence_number.clone().into()); window.set_msg5_svc(msg.service_name.clone().into()); window.set_msg5_x(msg_x as f32); window.set_msg5_y(msg_y as f32); window.set_msg5_angle(angle_deg as f32); window.set_msg5_vis(true); }
                    5 => { window.set_msg6_seq(msg.sequence_number.clone().into()); window.set_msg6_svc(msg.service_name.clone().into()); window.set_msg6_x(msg_x as f32); window.set_msg6_y(msg_y as f32); window.set_msg6_angle(angle_deg as f32); window.set_msg6_vis(true); }
                    6 => { window.set_msg7_seq(msg.sequence_number.clone().into()); window.set_msg7_svc(msg.service_name.clone().into()); window.set_msg7_x(msg_x as f32); window.set_msg7_y(msg_y as f32); window.set_msg7_angle(angle_deg as f32); window.set_msg7_vis(true); }
                    7 => { window.set_msg8_seq(msg.sequence_number.clone().into()); window.set_msg8_svc(msg.service_name.clone().into()); window.set_msg8_x(msg_x as f32); window.set_msg8_y(msg_y as f32); window.set_msg8_angle(angle_deg as f32); window.set_msg8_vis(true); }
                    8 => { window.set_msg9_seq(msg.sequence_number.clone().into()); window.set_msg9_svc(msg.service_name.clone().into()); window.set_msg9_x(msg_x as f32); window.set_msg9_y(msg_y as f32); window.set_msg9_angle(angle_deg as f32); window.set_msg9_vis(true); }
                    _ => {}
                }
            }
        }
    }
    
    // Update properties panel if object is selected
    if let Some(sel_id) = selected_id {
        if let Some(obj) = proj.objects.iter().find(|o| o.id == sel_id) {
            window.set_selected_obj_id(obj.id.clone().into());
            window.set_selected_obj_name(obj.name.clone().into());
            window.set_selected_obj_type(obj.type_name.clone().into());
            window.set_selected_obj_trait(obj.is_trait);
            window.set_selected_obj_x(obj.x as f32);
            window.set_selected_obj_y(obj.y as f32);
            window.set_has_selection(true);
        }
    } else {
        window.set_has_selection(false);
    }
}

fn clear_ui(window: &MainWindow) {
    for i in 0..10 {
        match i {
            0 => { window.set_obj1_visible(false); window.set_obj1_selected(false); }
            1 => { window.set_obj2_visible(false); window.set_obj2_selected(false); }
            2 => { window.set_obj3_visible(false); window.set_obj3_selected(false); }
            3 => { window.set_obj4_visible(false); window.set_obj4_selected(false); }
            4 => { window.set_obj5_visible(false); window.set_obj5_selected(false); }
            5 => { window.set_obj6_visible(false); window.set_obj6_selected(false); }
            6 => { window.set_obj7_visible(false); window.set_obj7_selected(false); }
            7 => { window.set_obj8_visible(false); window.set_obj8_selected(false); }
            8 => { window.set_obj9_visible(false); window.set_obj9_selected(false); }
            9 => { window.set_obj10_visible(false); window.set_obj10_selected(false); }
            _ => {}
        }
    }
    for i in 0..5 {
        match i {
            0 => window.set_assoc1_visible(false),
            1 => window.set_assoc2_visible(false),
            2 => window.set_assoc3_visible(false),
            3 => window.set_assoc4_visible(false),
            4 => window.set_assoc5_visible(false),
            _ => {}
        }
    }
    for i in 0..9 {
        match i {
            0 => window.set_msg1_vis(false),
            1 => window.set_msg2_vis(false),
            2 => window.set_msg3_vis(false),
            3 => window.set_msg4_vis(false),
            4 => window.set_msg5_vis(false),
            5 => window.set_msg6_vis(false),
            6 => window.set_msg7_vis(false),
            7 => window.set_msg8_vis(false),
            8 => window.set_msg9_vis(false),
            _ => {}
        }
    }
}

fn find_association_endpoints(proj: &RucdProject, assoc: &Association) -> Option<((f64, f64), (f64, f64))> {
    let src = proj.objects.iter().find(|o| o.id == assoc.source_id)?;
    let tgt = proj.objects.iter().find(|o| o.id == assoc.target_id)?;
    
    let src_center = (src.x + src.width / 2.0, src.y + src.height / 2.0);
    let tgt_center = (tgt.x + tgt.width / 2.0, tgt.y + tgt.height / 2.0);
    
    Some((src_center, tgt_center))
}

fn layout_objects_internal(project: &Rc<RefCell<RucdProject>>) {
    let mut proj_mut = project.borrow_mut();
    if proj_mut.objects.is_empty() {
        println!("rust-side:  No objects to layout");
        return;
    }
    
    let count = proj_mut.objects.len();
    let cols = ((count as f64).sqrt().ceil() as usize).max(2);
    let spacing_x = 280.0;
    let spacing_y = 200.0;
    
    for (i, obj) in proj_mut.objects.iter_mut().enumerate() {
        let col = i % cols;
        let row = i / cols;
        obj.x = 80.0 + (col as f64 * spacing_x);
        obj.y = 80.0 + (row as f64 * spacing_y);
    }
    
    println!("rust-side: ✓ Layout: {} objects in {} columns", count, cols);
}
