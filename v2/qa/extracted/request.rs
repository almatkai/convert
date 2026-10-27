impl Host {
pub async fn request(&self, p: &Arc<Process>, method: &str, params: Value) -> CoreResult {
        let capability = match method {
            "host.clipboard.read"
            | "host.clipboard.write"
            | "host.clipboard.writeImage"
            | "host.clipboard.writeFiles"
            | "host.clipboard.paste"
            | "host.clipboard.pasteText" => "clipboard",
            "host.shell.open" | "host.shell.openWithApp" | "host.shell.reveal" => "shell",
            "host.selection.read" => "selection",
            "host.window.visibility" | "host.window.suppressBlur" => "window",
            "host.dialog.confirm" => "dialog",
            "host.screen.capture" => "screen",
            "host.apps.list" => "apps",
            "host.settings.open" => "settings",
            "host.ai.ask" => "ai",
            "host.background.retain" | "host.background.release" => "background",
            "host.tray.set" | "host.tray.remove" => "tray",
            "host.notifications.show" | "host.notifications.remove" => "notifications",
            "host.safety.authorize" | "host.safety.record" => "safety",
            "host.launcher.publish" => "launcher",
            _ => return Err(format!("Unknown typed host capability: {method}")),
        };
        self.require(method, capability)?;
        if !params.is_object() {
            return Err("Capability parameters must be an object".into());
        }
        if !p.ready() {
            return Err(
                "Host UI/service calls are unavailable before initialization completes".into(),
            );
        }
        if !p.alive() {
            return Err("Extension generation is no longer active".into());
        }
        match method {
            "host.launcher.publish" => super::launcher::publish(p, &params),
            "host.background.retain"|"host.background.release"=>{
                let key=string(&params,"key")?;process_install::component(key)?;
                let retain=method.ends_with("retain");
                // Reserve capacity under the same lock as persistence. A failed
                // retain must never leave a durable activation without an owner.
                let mut r=self.resources.lock().map_err(|e|e.to_string())?;
                if retain&&r.leases.len()>=128&&!r.leases.contains(key){return Err("Too many background leases".into());}
                // Commit durable state BEFORE returning acknowledgment. Do not
                // erase durable records on crash/app shutdown; only release or
                // explicit uninstall removes them.
                background_update(&self.ctx,&self.id,key,if retain&&params["durable"]==true{Some(params["data"].clone())}else{None})?;
                if retain{r.leases.insert(key.into());}else{r.leases.remove(key);}
                p.changed.notify_one();Ok(json!({"retained":retain,"key":key}))
            },
            "host.window.suppressBlur"=>{let value=params["suppress"].as_bool().ok_or("Missing suppress boolean")?;let result=self.blur(value).await;p.changed.notify_one();result},
            "host.tray.set"=>{self.set_tray(p,params).await?;p.changed.notify_one();Ok(json!(true))},
            "host.tray.remove"=>{let id=string(&params,"resourceId")?;self.resources.lock().map_err(|e|e.to_string())?.trays.remove(id);p.changed.notify_one();Ok(json!(true))},
            "host.notifications.show"=>{
                let id=string(&params,"id")?;let scoped=self.scoped(id)?;
                {let r=self.resources.lock().map_err(|e|e.to_string())?;if r.notifications.len()>=128&&!r.notifications.contains(id){return Err("Too many notification resources".into());}}
                if params.get("action").is_some_and(|v|!v.is_null()){return Err("Notification actions require platform activation integration; not supported by this host revision".into());}
                let title=string(&params,"title")?;let body=string(&params,"body")?;
                // Own the resource before an asynchronous OS call can deliver it.
                // Cancellation/process death must still remove a late delivery.
                self.resources.lock().map_err(|e|e.to_string())?.notifications.insert(id.into());p.changed.notify_one();
                if let Err(error)=super::process_notifications::show(&scoped,title,body).await{
                    let _=super::process_notifications::remove(&scoped).await;
                    self.resources.lock().map_err(|e|e.to_string())?.notifications.remove(id);p.changed.notify_one();return Err(error);
                }Ok(json!(true))
            },
            "host.notifications.remove"=>{let id=string(&params,"id")?;super::process_notifications::remove(&self.scoped(id)?).await?;self.resources.lock().map_err(|e|e.to_string())?.notifications.remove(id);p.changed.notify_one();Ok(json!(true))},
            "host.clipboard.read"=>match params["format"].as_str(){None|Some("text")=>self.ctx.request("clipboard:read",vec![]).await,Some("image")=>read_clipboard_image().await,_=>Err("Unsupported clipboard format".into())},
            "host.clipboard.write"=>{
                let text=string(&params,"text")?;
                if let Some(html)=params.get("html"){if !html.is_string(){return Err("Clipboard HTML must be text".into());}self.ctx.request("core:platform:clipboard-write",vec![json!({"text":text,"html":html})]).await}
                else{self.ctx.request("clipboard:write",vec![json!(text)]).await}
            },
            "host.clipboard.writeImage"=>self.ctx.request("core:platform:clipboard-write",vec![json!({"file":params["file"].as_str().or_else(||params["path"].as_str()).ok_or("Missing image file")?})]).await,
            "host.clipboard.writeFiles"=>write_files(&params).await,
            "host.clipboard.paste"=>self.ctx.request("core:platform:paste",vec![]).await,
            "host.clipboard.pasteText"=>self.ctx.request("core:platform:paste-text",vec![json!(string(&params,"text")?)]).await,
            "host.shell.open"=>self.ctx.request("shell:open",vec![json!(string(&params,"target")?)]).await,
            "host.shell.openWithApp"=>self.ctx.request("core:platform:open-with-app",vec![json!({"path":string(&params,"path")?,"appPath":string(&params,"appPath")?})]).await,
            "host.shell.reveal"=>self.ctx.request("core:platform:reveal-file",vec![json!(string(&params,"path")?)]).await,
            "host.selection.read"=>match params["kind"].as_str(){None|Some("text")=>self.ctx.request("core:platform:selected-text",vec![]).await,Some("files")=>selected_files().await,_=>Err("Unsupported selection kind".into())},
            "host.window.visibility"=>self.ctx.request("core:platform:visibility",vec![json!(params["visible"].as_bool().ok_or("Missing visible boolean")?)]).await,
            "host.dialog.confirm"=>self.ctx.request("core:platform:confirm",vec![params]).await,
            "host.screen.capture"=>{
                let value=self.ctx.request("core:platform:screenshot",vec![json!({"maxWidth":params["maxWidth"]})]).await?;
                if value["mimeType"]!="image/png"||!value["data"].as_str().is_some_and(|s|s.len()<=15*1024*1024){return Err("Capture image exceeds protocol limit or has invalid shape".into());}Ok(value)
            },
            "host.apps.list"=>self.ctx.request("core:apps:list",vec![]).await,
            "host.settings.open"=>self.ctx.request("core:platform:open-settings",vec![json!(params["section"].as_str().or_else(||params["surface"].as_str()).unwrap_or("settings-extensions"))]).await,
            "host.ai.ask"=>{
                let prompt=string(&params,"prompt")?;
                let options=params.get("options").cloned().unwrap_or(json!({}));
                if !options.is_object(){return Err("AI options must be an object".into());}
                self.ctx.request("core:ai:extension-ask",vec![json!(prompt),options]).await
            },
            "host.safety.authorize"|"host.safety.record"=>{
                let operation=string(&params,"operation")?;
                if operation.len()>128{return Err("Safety operation too long".into());}
                let mut context=params["context"].as_object().cloned().ok_or("Safety context must be an object")?;
                context.insert("extensionId".into(),json!(self.id));
                let mut args=vec![json!(operation),Value::Object(context)];
                if method.ends_with("record"){args.push(params["result"].clone());}
                self.ctx.request(if method.ends_with("authorize"){"core:safety:authorize"}else{"core:safety:record"},args).await
            },
            _=>unreachable!()
        }
    }
}
