const board = document.getElementById("board");

const position = [
    ["♜", "♞", "♝", "♛", "♚", "♝", "♞", "♜"],
    ["♟", "♟", "♟", "♟", "♟", "♟", "♟", "♟"],
    ["", "", "", "", "", "", "", ""],
    ["", "", "", "", "", "", "", ""],
    ["", "", "", "", "", "", "", ""],
    ["", "", "", "", "", "", "", ""],
    ["♙", "♙", "♙", "♙", "♙", "♙", "♙", "♙"],
    ["♖", "♘", "♗", "♕", "♔", "♗", "♘", "♖"]
];

let selectedRow;
let selectedCol;

function toUci(row, col) {
    const file = String.fromCharCode("a".charCodeAt(0) + col);
    const rank = 8 - row;

    return file + rank;
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

            square.textContent = position[row][col];

            square.addEventListener("click", async function() {
                if (selectedRow === undefined) {
                    if (position[row][col] === "") {
                        return;
                    }

                    selectedRow = row;
                    selectedCol = col;

                    square.classList.add("selected");

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
                        position[row][col] = position[selectedRow][selectedCol];
                        position[selectedRow][selectedCol] = "";

                        selectedRow = undefined;
                        selectedCol = undefined;

                        drawBoard();

                        if (result === "Checkmate") {
                            alert("Checkmate!");
                        } else if (result === "Draw") {
                            alert("Draw!");
                        }
                    }

                    else if (result === "Illegal") {
                        alert("Illegal move.");

                        selectedRow = undefined;
                        selectedCol = undefined;

                        drawBoard();
                    }

                    else {
                        console.error("Unknown server response:", result);
                    }
                } catch (error) {
                    console.error("Failed to send move:", error);

                    selectedRow = undefined;
                    selectedCol = undefined;

                    drawBoard();
                }
            });

            board.appendChild(square);
        }
    }
}

drawBoard();
