const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { appWindow } = window.__TAURI__.window;

const autosaveEl = document.querySelector("#autosave-upload");
const uploadsEl = document.querySelector("#uploads");
const downloadsEl = document.querySelector("#downloads");
const goEl = document.querySelector('#go');
const resultEl = document.querySelector('#result');

// scut_core::interface::Prediction
let currentPrediction;

function readForm(form) {
    return [...form.querySelectorAll('input[type="checkbox"]')].filter(el => el.checked).map(el => el.dataset.save);
}

async function download() {
    let savesIter = readForm(downloadsEl);

    try {
        return await invoke("download", { items: [...savesIter] });
    } catch(e) {
        return `Error: ${e}`;
    }
}

async function upload() {
    let autosave = readForm(autosaveEl);
    let savesIter = readForm(uploadsEl);

    if (autosave.length === 0) {
        autosave = null;
    } else {
        autosave = autosave[0];
    }

    let items = [...savesIter];

    if (!autosave && items.length === 0) {
        return false;
    }

    try {
        return await invoke("upload", { autosave, items });
    } catch(e) {
        return `Error: ${e}`;
    }
}

async function uploadAndDownload(e) {
    e.preventDefault();

    startLoad();

    const uploads = upload();
    const downloads = download();

    const results = [];
    let result = await uploads;
    if (result) {
        results.push(result);
    }
    result = await downloads;
    if (result) {
        results.push(result);
    }

    await refresh(results);
}

function predictionIsEmpty() {
    return currentPrediction && !(currentPrediction.autosave.status === 'Ready' || currentPrediction.uploads.length > 0 || currentPrediction.downloads.length > 0)
}

function startLoad() {
    document.body.classList.add('loading');
    goEl.disabled = true;
}

function stopLoad() {
    document.body.classList.remove('loading');
    goEl.disabled = false;
}
async function refresh(results) {

    let prediction = await invoke('predict');
    currentPrediction = prediction;
    const now = new Date().toLocaleTimeString();

    let checked = `Checked: ${now}`;

    if (results) {
        results.push(checked);
    } else {
        results = [checked];
    }

    render(prediction, results);
}

function render({
    autosave: {
        save: [autosave, reason],
    },
    uploads,
    downloads,
}, results) {
    startLoad();

    // remove all previous save items
    autosaveEl.replaceChildren([]);
    uploadsEl.replaceChildren([]);
    downloadsEl.replaceChildren([]);

    if (autosave.status == 'ready') {
        autosaveEl.appendChild(renderSaveItem(autosave));
    }

    uploads.forEach(save => {
        uploadsEl.appendChild(renderSaveItem(save));
    });

    downloads.forEach(save => {
        downloadsEl.appendChild(renderSaveItem(save));
    });

    if (uploads.length === 0 && status === 'NotReady') {
        let warning = document.createElement('p');
        warning.textContent = 'No saves to upload';
        uploadsEl.appendChild(warning);
    }

    if (downloads.length === 0) {
        let warning = document.createElement('p');
        warning.textContent = 'No saves to download';
        downloadsEl.appendChild(warning);
    }

    results = results.map(result => renderResultItem(result));
    resultEl.replaceChildren([]);
    results.forEach(result => {
        resultEl.appendChild(result);
    });

    stopLoad();

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
    let label = clone.querySelector("label");
    let input = clone.querySelector("input");
    if (autosave) {
        label.childNodes[0].nodeValue = `${side} ${number} (Autosave)`;
        input.dataset.save = `${side} ${number}`;
    } else {
        label.childNodes[0].nodeValue = `${side} ${player} ${number}${part}`;
        input.dataset.save = `${side} ${player} ${number}${part}`;
    }
    return clone;
}

function renderResultItem(result) {
    const template = document.querySelector("#result-item-template");
    const clone = template.content.cloneNode(true);

    let item = clone.querySelector("li");
    item.childNodes[0].nodeValue = result;
    return clone;
}

await appWindow.listen('trayOpen', e => refresh());
goEl.addEventListener('click', uploadAndDownload);
