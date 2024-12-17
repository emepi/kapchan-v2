const kapchanState = {
    current_captcha: 0,
};

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
        kapchanState.current_captcha = captcha_data.id;
    });
}

const submitPost = () => {
    const pf = document.getElementById("posting-form");
    const data = new FormData(pf)

    if(data.has("captcha")) {
        data.append("captcha_id", kapchanState.current_captcha.toString())
    }

    fetch(window.location.href, {
        method: "POST",
        body: data,
    })
};