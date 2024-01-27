const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { appWindow } = window.__TAURI__.window;

let outputEl = document.querySelector("#output");
const form = document.querySelector("#save-item-list");
form.addEventListener("submit", onFormSubmit);

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
    outputEl.textContent = response;
}

await appWindow.listen('trayOpen', () => {
    invoke('predict')
        .then(prediction => {
            currentPrediction = prediction;
            console.log(prediction);
            render(prediction);
        })
})

function onFormSubmit(e) {
    e.preventDefault();
    upload(e.target);
}

function render({
    autosave: {
        save: [save, reason],
    },
    uploads,
    downloads,
}) {
    const out = document.querySelector("#save-item-list");
    // remove all previous save items
    out.replaceChildren([]);

    if (save.status == 'ready') {
        out.appendChild(renderSaveItem(save));
    }

    uploads.forEach(save => {
        out.appendChild(renderSaveItem(save));
    });

    downloads.forEach(save => {
        out.appendChild(renderSaveItem(save));
    });
}


function renderSaveItem({
    player,
    turn: {
        side,
        number,
    },
    part,
}) {
    const template = document.querySelector("#save-item-template");
    const clone = template.content.cloneNode(true);
    if (!player) {
        player = '';
    }
    if (!number) {
        number = '';
    }
    if (!part) {
        part = '';
    }
    clone.querySelector("label").childNodes[0].nodeValue = `${side} ${player} ${number}${part}`;
    return clone;
}