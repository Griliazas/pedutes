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
//amelija ar cia tu??
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

const avatarOptions = document.querySelectorAll(".avatar-option");
const profileAvatar = document.getElementById("profile-avatar");
const avatarDropdown = document.querySelector(".avatar-dropdown");
const avatarStatus = document.getElementById("avatar-status");

avatarOptions.forEach(option => {
    option.addEventListener("click", async () => {
        const avatar = option.dataset.avatar;

        avatarOptions.forEach(btn => btn.disabled = true);
        avatarStatus.textContent = "";

        try {
            const response = await fetch("/avatar", {
                method:"POST",
                headers: {
                    "Content-Type":
                    "application/x-www-form-urlencoded"
                },
                body: new URLSearchParams({
                    avatar: avatar
                })
            });

            if (!response.ok) {
                avatarStatus.textContent = await response.text();
                return;
            }

            profileAvatar.src = `/res/avatars/pic${avatar}.png`;
            avatarDropdown.open = false;
        } catch (error) {
            avatarStatus.textContent = "Could not update avatar :(";
            console.error(error);
        } finally {
            avatarOptions.forEach(btn => btn.disabled = false);
        }
    });
});

//filtravimui

const filterButtons = document.querySelectorAll(".filter-btn");
const postBoxes = document.querySelectorAll(".posts-feed .post-box");

if (filterButtons.length > 0) {
    filterButtons.forEach(button => {
        button.addEventListener("click", () => {
            filterButtons.forEach(btn => btn.classList.remove("active"));
            button.classList.add("active");

            const selectedTag = button.dataset.filter;

            postBoxes.forEach(post => {
                const postTagEl = post.querySelector(".post-tag");
                const postTag = postTagEl ? postTagEl.textContent.trim() : "";

                if (selectedTag === "all" || postTag === selectedTag) {
                    post.style.display = "";
                } else {
                    post.style.display = "none";
                }
            });
        });
    });
}