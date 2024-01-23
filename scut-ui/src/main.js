const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { appWindow } = window.__TAURI__.window;

let uploadFormEl;
let outputEl;

// scut_core::interface::Prediction
let currentPrediction;

// scut_core::interface::Selection
let currentSelection;

async function upload(form) {
    let formData = new FormData(form);
    for (let item of formData.keys()) {
        console.log(item);
    }
    let response = await invoke("upload", { items: [...formData.keys()] });
    console.log("got response:", response);
    outputEl.text = response;
}

await appWindow.listen('trayOpen', () => {
    invoke('prediction/request')
        .then(prediction => {
            currentPrediction = prediction;
            render(prediction);
        })
})

window.addEventListener("DOMContentLoaded", () => {
    uploadFormEl = document.querySelector("#upload-form");
    outputEl = document.querySelector("#output");
    document.querySelector("#upload-form").addEventListener("submit", (e) => {
        e.preventDefault();
        upload(e.target);
    });
});

function render({
    autosave,
    uploads,
    downloads,
}) {
    const template = document.querySelector("#save-item-template");

    if (autosave.status == 'ready') {
        
    }

}

function renderSaveItem({
    player,
    turn,
    part,
}) {
    const template = document.querySelector("#save-item-template");
    const clone = template.content.cloneNode(true);
    clone.querySelector("label").textContent = `${player}`
}