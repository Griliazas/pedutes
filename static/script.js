
const formReg = document.getElementById("register-form");
const formLog = document.getElementById("login-form");
//Cia registracijai
if (formReg) {
    formReg.addEventListener("submit", async (e) => {
        e.preventDefault();

        try {
            const response = await fetch(formReg.action, {
                method: "POST",
                headers: {
                    "Content-Type": "application/x-www-form-urlencoded"
                },
                body: new URLSearchParams(new FormData(formReg))
            });

            if (!response.ok) {
                const message = await response.text();
                showNotification(message, "error");
                return;
            }

            showNotification("Registration successful!", "success");

            setTimeout(() => {
                window.location.href = "/login";
            }, 1500);

        } catch (error) {
            showNotification("Woopsie kazkas susipiso", "error");
            console.error(error);
        }
    });
}
//Cia loginui
if (formLog) {
    formLog.addEventListener("submit", async (e) => {
        e.preventDefault();

        try {
            const response = await fetch(formLog.action, {
                method: "POST",
                headers: {
                    "Content-Type": "application/x-www-form-urlencoded"
                },
                body: new URLSearchParams(new FormData(formLog))
            });

            if (!response.ok) {
                const message = await response.text();
                showNotification(message, "error");
                return;
            }

            showNotification("Login successful!", "success");

            setTimeout(() => {
                window.location.href = "/";
            }, 1500);

        } catch (error) {
            showNotification("Woopsie kazkas susipiso", "error");
            console.error(error);
        }
    });
}
//Cia tsg notifikacija
function showNotification(message, type) {
    const notification = document.getElementById("notification");

    if (!notification) return;

    notification.textContent = message;
    notification.className = `notification ${type} show`;

    setTimeout(() => {
        notification.classList.remove("show");
    }, 3000);
}

const textarea = document.querySelector(".textbox");
const current = document.getElementById("current");

if (textarea && current) {
    textarea.addEventListener("input", () => {
        const characterCount = textarea.value.length;
        current.textContent = characterCount;
    });
}
