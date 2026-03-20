// Use global Tauri API for vanilla compatibility
const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

let selectedFiles = [];
let saveDir = "";

console.log("Image Converter Pro: JS Loaded");

const fileList = document.getElementById("file-list");
const addFilesBtn = document.getElementById("add-files");
const addFolderBtn = document.getElementById("add-folder");
const clearListBtn = document.getElementById("clear-list");
const targetFormatSelect = document.getElementById("target-format");
const saveDirInput = document.getElementById("save-dir");
const browseSaveDirBtn = document.getElementById("browse-save-dir");
const openSaveDirBtn = document.getElementById("open-save-dir");
const convertBtn = document.getElementById("convert-btn");

openSaveDirBtn.addEventListener("click", async () => {
  const saveDir = saveDirInput.value;
  if (saveDir) {
    try {
      await invoke("open_folder", { path: saveDir });
    } catch (err) {
      console.error("Failed to open directory:", err);
    }
  }
});

async function updateFileList() {
  if (selectedFiles.length === 0) {
    fileList.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-images"></i>
                <p>Select images to start converting</p>
            </div>
        `;
    return;
  }

  fileList.innerHTML = "";
  selectedFiles.forEach((file, index) => {
    const card = document.createElement("div");
    card.className = "file-card";
    const safeId = "file-" + btoa(file).replace(/=/g, "").substr(-10);
    card.id = safeId;

    const name = file.split(/[\\/]/).pop();
    const ext = file.split(".").pop().toUpperCase();

    card.innerHTML = `
            <div class="preview-container col-thumb">
                <i class="fas fa-spinner fa-spin" style="color: var(--text-secondary)"></i>
            </div>
            <div class="file-info">
                <div class="file-name col-name" title="${file}">${name}</div>
                <div class="file-ext col-format">${ext}</div>
                <div class="status-badge status-pending col-status">Pending</div>
            </div>
            <div class="btn-remove col-action" data-index="${index}" title="Remove from list">
                <i class="fas fa-times"></i>
            </div>
        `;
    fileList.appendChild(card);

    // Load preview asynchronously
    invoke("get_image_preview", { path: file })
      .then((metadata) => {
        const previewContainer = card.querySelector(".preview-container");
        previewContainer.innerHTML = `<img src="${metadata.preview}" alt="${metadata.name}" />`;
      })
      .catch((err) => {
        console.error("Preview error:", err);
        const previewContainer = card.querySelector(".preview-container");
        previewContainer.innerHTML = `<i class="fas fa-exclamation-triangle" style="color: var(--error-color)"></i>`;
      });
  });

  // Add removal listeners
  fileList.querySelectorAll(".btn-remove").forEach(btn => {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      const index = parseInt(btn.getAttribute("data-index"));
      selectedFiles.splice(index, 1);
      updateFileList();
    });
  });
}

addFilesBtn.addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "bmp", "dds", "ddj", "tif", "tiff", "gif", "ico"]
      }]
    });
    if (selected) {
      selectedFiles = [...new Set([...selectedFiles, ...selected])];
      updateFileList();
    }
  } catch (err) {
    console.error("Add files error:", err);
    alert("Error opening file dialog: " + err);
  }
});

addFolderBtn.addEventListener("click", async () => {
  try {
    const selected = await open({
      directory: true,
    });
    if (selected) {
      const files = await invoke("read_folder", { path: selected });
      selectedFiles = [...new Set([...selectedFiles, ...files])];
      updateFileList();
    }
  } catch (err) {
    console.error("Add folder error:", err);
    alert("Error opening folder dialog: " + err);
  }
});

clearListBtn.addEventListener("click", () => {
  selectedFiles = [];
  updateFileList();
});

browseSaveDirBtn.addEventListener("click", async () => {
  try {
    const selected = await open({
      directory: true,
    });
    if (selected) {
      saveDir = selected;
      saveDirInput.value = selected;
    }
  } catch (err) {
    console.error("Browse dir error:", err);
  }
});

convertBtn.addEventListener("click", async () => {
  if (selectedFiles.length === 0) return;
  if (!saveDir) {
    alert("Please select a save directory first!");
    return;
  }

  convertBtn.disabled = true;
  convertBtn.innerHTML = `<i class="fas fa-spinner fa-spin"></i> Converting...`;

  const targetFormat = targetFormatSelect.value;
  const cards = document.querySelectorAll(".file-card");

  for (let i = 0; i < selectedFiles.length; i++) {
    const file = selectedFiles[i];
    const card = cards[i];
    if (!card) continue;

    const statusEl = card.querySelector(".status-badge");
    const sourceExt = file.split(".").pop().toLowerCase();
    const targetExt = targetFormat.toLowerCase();

    // Normalization for JPG/JPEG
    const isJpgMatch = (sourceExt === "jpg" || sourceExt === "jpeg") && (targetExt === "jpg" || targetExt === "jpeg");
    const isSameFormat = sourceExt === targetExt || isJpgMatch;

    if (isSameFormat) {
      statusEl.textContent = "Skipped";
      statusEl.className = "status-badge status-success";
      continue;
    }

    statusEl.textContent = "Converting...";
    statusEl.className = "status-badge status-converting";

    try {
      const result = await invoke("convert_image", {
        path: file,
        targetFormat,
        saveDir
      });

      if (result.success) {
        statusEl.textContent = "Success";
        statusEl.className = "status-badge status-success";
      } else {
        statusEl.textContent = "Error";
        statusEl.className = "status-badge status-error";
        console.error("Conversion error result:", result.error);
      }
    } catch (err) {
      statusEl.textContent = "Failed";
      statusEl.className = "status-badge status-error";
      console.error("Conversion catch error:", err);
    }
  }

  convertBtn.disabled = false;
  convertBtn.innerHTML = `<i class="fas fa-bolt"></i> Convert All`;
});
