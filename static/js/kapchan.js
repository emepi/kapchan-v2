const logout = (event) => {
    fetch(new Request("/logout", {
            method: "POST",
        })
    )
    .then(() => location.reload());
};

const openPosting = () => {
    const ps = document.getElementById("posting-screen");
    ps.hidden = false;
}

const closePosting = () => {
    const ps = document.getElementById("posting-screen");
    ps.hidden = true;
}

const fetchCaptcha = () => {
    const cc = document.getElementById("captcha-container");

    fetch(new Request("/captcha", {
            method: "GET",
        })
    )
    .then(res => res.json())
    .then(captcha_data => {
        let captcha = document.createElement("img");
        captcha.setAttribute("src", "data:image/jpg;base64," + captcha_data.captcha);

        cc.innerHTML = "";
        cc.appendChild(captcha);
    });
}