use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
}

async fn fetch_users() -> Result<Vec<User>, String> {
    let res = reqwest::get("https://jsonplaceholder.typicode.com/users")
        .await
        .map_err(|e| e.to_string())?;
        
    let users = res.json::<Vec<User>>()
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(users)
}
#[component]
pub fn Home() -> impl IntoView {
    let users_data = LocalResource::new(|| async move { fetch_users().await });
    view! {
        <div class="container mx-auto max-w-screen-xl">

            <h1 class="text-3xl font-bold text-blue-600 text-center ">"Satellite"</h1>

            // <div class="mt-8">
            //     <h2 class="text-2xl font-bold mb-4">"Benutzerliste"</h2>
                
            //     <Suspense fallback=|| view! { <p class="text-gray-500">"Lade Daten..."</p> }>
            //         {move || {
            //             users_data.get().map(|result| {
            //                 match result.as_ref() {
            //                     Ok(users) => view! {
            //                         <ul class="space-y-2">
            //                             {users.iter().map(|user| view! {
            //                                 <li class="p-2 bg-blue-50 rounded shadow">
            //                                     {user.name.clone()}
            //                                 </li>
            //                             }).collect::<Vec<_>>()}
            //                         </ul>
            //                     }.into_any(),
            //                     Err(e) => view! {
            //                         <p class="text-red-500">"Fehler beim Laden: " {e.clone()}</p>
            //                     }.into_any(),
            //                 }
            //             })
            //         }}
            //     </Suspense>
            // </div>
        </div>
    }
}