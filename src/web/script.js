const board = document.getElementById("board");
const statusMessage = document.getElementById("status");

let position = [];
let selectedRow;
let selectedCol;

const unicodePieces = {
    "K": "♚",
    "Q": "♛",
    "R": "♜",
    "B": "♝",
    "N": "♞",
    "P": "♟",

    "k": "♚",
    "q": "♛",
    "r": "♜",
    "b": "♝",
    "n": "♞",
    "p": "♟"
};

function showStatus(message) {
    statusMessage.textContent = message;
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

function drawBoard() {
    board.innerHTML = "";

    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
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

            if (selectedRow === row && selectedCol === col) {
                square.classList.add("selected");
            }

            square.addEventListener("click", async function() {
                if (selectedRow === undefined) {
                    if (position[row][col] === "") {
                        return;
                    }

                    selectedRow = row;
                    selectedCol = col;

                    drawBoard();
                    return;
                }

                if (selectedRow === row && selectedCol === col) {
                    selectedRow = undefined;
                    selectedCol = undefined;

                    drawBoard();
                    return;
                }

                const from = toUci(selectedRow, selectedCol);
                const to = toUci(row, col);
                const move = from + to;

                try {
                    const result = await sendMove(move);

                    if (
                        result === "Ok" ||
                        result === "Checkmate" ||
                        result === "Draw"
                    ) {
                        selectedRow = undefined;
                        selectedCol = undefined;

                        await getBoard();
                        drawBoard();

                        if (result === "Checkmate") {
                            showStatus("Checkmate!");
                        } else if (result === "Draw") {
                            showStatus("Draw!");
                        } else {
                            showStatus("");
                        }

                        showStatus("Bot is thinking...");

                        try {
                            const botResult = await sendBotMove();

                            await getBoard();
                            drawBoard();

                            if (botResult === "Checkmate") {
                                showStatus("Checkmate!");
                            } else if (botResult === "Draw") {
                                showStatus("Draw!");
                            } else if (botResult === "Ok") {
                                showStatus("");
                            } else {
                                console.error("Unknown bot response:", botResult);
                                showStatus("");
                            }
                        } catch (error) {
                            console.error("Failed to make bot move:", error);
                            showStatus("Failed to make bot move.");
                        }
                    }

                    else if (result === "Illegal") {
                        showStatus("Illegal move.");

                        selectedRow = undefined;
                        selectedCol = undefined;

                        drawBoard();
                    }

                    else {
                        console.error("Unknown server response:", result);

                        showStatus("");

                        selectedRow = undefined;
                        selectedCol = undefined;

                        await getBoard();
                        drawBoard();
                    }
                } catch (error) {
                    console.error("Failed to send move:", error);

                    selectedRow = undefined;
                    selectedCol = undefined;

                    try {
                        await getBoard();
                        drawBoard();
                    } catch (boardError) {
                        console.error("Failed to get board:", boardError);
                    }
                }
            });

            board.appendChild(square);
        }
    }
}

async function init() {
    try {
        await getBoard();
        drawBoard();
    } catch (error) {
        console.error("Failed to initialize board:", error);
    }
}

async function resetGame() {
    try {
        const response = await fetch("/reset", {
            method: "POST"
        });

        if (!response.ok) {
            throw new Error(`Failed to reset game: ${response.status}`);
        }

        selectedRow = undefined;
        selectedCol = undefined;

        await getBoard();
        drawBoard();

        showStatus("");
    } catch (error) {
        console.error("Failed to reset game:", error);
        showStatus("Failed to reset game.");
    }
}

document.addEventListener("keydown", function(event) {
    if (event.key.toLowerCase() === "r") {
        resetGame();
    }
});

init();
