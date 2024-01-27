const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { appWindow } = window.__TAURI__.window;

const uploadsEl = document.querySelector("#uploads");
const downloadsEl = document.querySelector("#downloads");
const goEl = document.querySelector('#go');
const resultEl = document.querySelector('#result');

// scut_core::interface::Prediction
let currentPrediction;

function readForm(form) {
    return [...form.querySelectorAll('input[checked] + label')].map(el => el.textContent);
}

async function download() {
    let savesIter = readForm(downloadsEl);
    let response = await invoke("download", { items: [...savesIter] });
    console.log("download response: ", response);
}

async function upload() {
    let savesIter = readForm(uploadsEl);
    let response = await invoke("upload", { items: [...savesIter] });
    console.log("upload response: ", response);
}

async function refresh() {
    let prediction = await invoke('predict');
    currentPrediction = prediction;
    render(prediction);
}

async function uploadAndDownload(e) {
    e.preventDefault();
    const uploads = upload();
    const downloads = download();

    const results = {
        uploadResult: await uploads,
        downloadResult: await downloads,
    };
    await refresh();
}

function predictionIsEmpty() {
    return currentPrediction && !(currentPrediction.autosave.status === 'Ready' || currentPrediction.uploads.length > 0 || currentPrediction.downloads.length > 0)
}

await appWindow.listen('trayOpen', refresh);
goEl.addEventListener('click', uploadAndDownload);

function render({
    autosave: {
        save: [autosave, reason],
    },
    uploads,
    downloads,
}) {
    // remove all previous save items
    uploadsEl.replaceChildren([]);
    downloadsEl.replaceChildren([]);

    if (autosave.status == 'ready') {
        uploadsEl.appendChild(renderSaveItem(autosave));
    }

    uploads.forEach(save => {
        uploadsEl.appendChild(renderSaveItem(save));
    });

    downloads.forEach(save => {
        downloadsEl.appendChild(renderSaveItem(save));
    });

    if (uploads.length === 0 && autosave.status !== 'ready') {
        let warning = document.createElement('p');
        warning.textContent = 'No saves to upload';
        uploadsEl.appendChild(warning);
    }

    if (downloads.length === 0) {
        let warning = document.createElement('p');
        warning.textContent = 'No saves to download';
        downloadsEl.appendChild(warning);
    }

    if (predictionIsEmpty()) {
        goEl.disabled = true;
        goEl.textContent = 'Nothing to do 💤';
    } else {
        goEl.disabled = false;
        goEl.textContent = 'Go';
    }
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
    let autosave = false;
    if (!player) {
        player = '';
        autosave = true;
    }
    if (!number) {
        number = '';
    }
    if (!part) {
        part = '';
    }
    if (autosave) {
        clone.querySelector("label").childNodes[0].nodeValue = `Autosave (${side} ${number})`;
    } else {
        clone.querySelector("label").childNodes[0].nodeValue = `${side} ${player} ${number}${part}`;
    }
    return clone;
}
