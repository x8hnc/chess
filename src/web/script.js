const board = document.getElementById("board");
const statusMessage = document.getElementById("status");
const promotionMenu = document.getElementById("promotion");

let position = [];
let selectedSquare = null;
let turn = null;
let choosingPromotion = false;

const unicodePieces = {
    K: "♚",
    Q: "♛",
    R: "♜",
    B: "♝",
    N: "♞",
    P: "♟",
    k: "♚",
    q: "♛",
    r: "♜",
    b: "♝",
    n: "♞",
    p: "♟"
};


function showStatus(message) {
    statusMessage.textContent = message;
}

function clearSelection() {
    selectedSquare = null;
}

function drawBoard() {
    board.innerHTML = "";

    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const square = createSquare(row, col);
            board.appendChild(square);
        }
    }
}

function createSquare(row, col) {
    const square = document.createElement("div");

    square.classList.add("square");

    if ((row + col) % 2 === 0) {
        square.classList.add("light");
    } else {
        square.classList.add("dark");
    }

    const piece = position[row][col];

    if (piece !== "") {
        square.textContent = unicodePieces[piece] || piece;

        square.classList.add(
            piece === piece.toUpperCase()
                ? "white-piece"
                : "black-piece"
        );
    }

    if (
        selectedSquare &&
        selectedSquare.row === row &&
        selectedSquare.col === col
    ) {
        square.classList.add("selected");
    }

    square.addEventListener("click", () => {
        handleSquareClick(row, col);
    });

    return square;
}

function toUci(row, col) {
    const file = String.fromCharCode("a".charCodeAt(0) + col);
    const rank = 8 - row;

    return file + rank;
}

function uciToPosition(square) {
    const col = square.charCodeAt(0) - "a".charCodeAt(0);
    const row = 8 - parseInt(square[1]);

    return { row, col };
}

async function getBoard() {
    const response = await fetch("/board");

    if (!response.ok) {
        throw new Error(`Failed to get board: ${response.status}`);
    }

    const text = await response.text();

    position = text.trim().split("\n").map(row =>
        [...row].map(piece => piece === "." ? "" : piece)
    );

    if (
        position.length !== 8 ||
        position.some(row => row.length !== 8)
    ) {
        throw new Error("Invalid board received from server");
    }
}

async function getTurn() {
    const response = await fetch("/turn");

    if (!response.ok) {
        throw new Error(`Failed to get turn: ${response.status}`);
    }

    turn = (await response.text()).trim();

    if (turn !== "player" && turn !== "bot") {
        throw new Error(`Invalid turn received from server: ${turn}`);
    }
}

async function sendMove(move) {
    const response = await fetch("/move", {
        method: "POST",
        headers: {
            "Content-Type": "text/plain"
        },
        body: move
    });

    return (await response.text()).trim();
}

async function sendBotMove() {
    const response = await fetch("/bot_move", {
        method: "POST"
    });

    if (!response.ok) {
        throw new Error(`Failed to make bot move: ${response.status}`);
    }

    return (await response.text()).trim();
}

async function resetServerGame() {
    const response = await fetch("/reset", {
        method: "POST"
    });

    if (!response.ok) {
        throw new Error(`Failed to reset game: ${response.status}`);
    }
}

function choosePromotion() {
    choosingPromotion = true;
    promotionMenu.classList.remove("hidden");

    return new Promise(resolve => {
        const buttons = promotionMenu.querySelectorAll("button");

        buttons.forEach(button => {
            button.onclick = () => {
                choosingPromotion = false;
                promotionMenu.classList.add("hidden");

                resolve(button.dataset.piece);
            };
        });
    });
}

async function handleSquareClick(row, col) {
    if (turn !== "player" || choosingPromotion) {
        return;
    }

    if (!selectedSquare) {
        selectSquare(row, col);
        return;
    }

    if (
        selectedSquare.row === row &&
        selectedSquare.col === col
    ) {
        clearSelection();
        drawBoard();
        return;
    }

    await attemptMove(row, col);
}

function selectSquare(row, col) {
    if (position[row][col] === "") {
        return;
    }

    selectedSquare = { row, col };
    drawBoard();
}

async function attemptMove(row, col) {
    const from = toUci(selectedSquare.row, selectedSquare.col);
    const to = toUci(row, col);

    const piece = position[selectedSquare.row][selectedSquare.col];

    let move = from + to;

    const isPromotion =
        piece.toLowerCase() === "p" &&
        (row === 0 || row === 7);

    if (isPromotion) {
        const promotionPiece = await choosePromotion();
        move += promotionPiece;
    }

    await playMove(move);
}

async function playMove(move) {
    try {
        const result = await sendMove(move);

        clearSelection();

        if (result === "Illegal") {
            showStatus("Illegal move.");
            drawBoard();
            return;
        }

        if (
            result !== "Ok" &&
            result !== "Checkmate" &&
            result !== "Draw"
        ) {
            console.error("Unknown server response:", result);
            showStatus("");
            await refreshBoard();
            return;
        }

        await refreshBoard();

        if (result === "Checkmate") {
            showStatus("Checkmate!");
            return;
        }

        if (result === "Draw") {
            showStatus("Draw!");
            return;
        }

        await handleTurn();
    } catch (error) {
        console.error("Failed to send move:", error);
    }
}

async function handleTurn() {
    await getTurn();

    if (turn === "bot") {
        await playBotMove();
    }
}

async function playBotMove() {
    showStatus("Bot is thinking...");

    try {
        const result = await sendBotMove();

        await refreshBoard();

        if (result === "Checkmate") {
            showStatus("Checkmate!");
            return;
        }

        if (result === "Draw") {
            showStatus("Draw!");
            return;
        }

        if (result !== "Ok") {
            console.error("Unknown bot response:", result);
            return;
        }
        
        showStatus("");
        await handleTurn();
    } catch (error) {
        console.error("Failed to make bot move:", error);
        showStatus("Failed to make bot move.");
    }
}

async function refreshBoard() {
    await getBoard();
    drawBoard();
}

async function resetGame() {
    try {
        await resetServerGame();

        clearSelection();
        showStatus("");

        await refreshBoard();
        await handleTurn();
    } catch (error) {
        console.error("Failed to reset game:", error);
        showStatus("Failed to reset game.");
    }
}

async function init() {
    try {
        await refreshBoard();
        await handleTurn();
    } catch (error) {
        console.error("Failed to initialize board:", error);
        showStatus("Failed to load game.");
    }
}

document.addEventListener("keydown", event => {
    if (event.key.toLowerCase() === "r") {
        resetGame();
    }
});


init();
