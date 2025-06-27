const kapchanState = {
    current_captcha: 0,
};

const logout = (event) => {
    fetch(new Request("/logout", {
            method: "POST",
        })
    )
    .then(() => location.replace("/"));
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
    const data = new FormData(pf);

    if(data.has("captcha")) {
        data.append("captcha_id", kapchanState.current_captcha.toString());
    }

    fetch(window.location.href, {
        method: "POST",
        body: data,
    })
    .then((res) => {
        if (res.ok) {
            pf.reset();
            location.reload();
        } else if (res.status == 403) {
            res.json()
            .then(json => {
                let errContainer = document.getElementById("err-container");
                let err = document.createTextNode(json.error);
                errContainer.innerHTML = "";
                errContainer.appendChild(err);
            });
        }
    })
};

const enlargeImage = (container_id, image_id) => {
    const image_container = document.getElementById(container_id);
    image_container.children[0].src = "/files/" + image_id;

    if (image_container.classList.contains("image-container")) {
        image_container.classList.replace("image-container", "image-container-large");
    } else {
        image_container.classList.replace("image-container-large", "image-container");
    }
}

const openMobileMenu = () => {
  const mobileMenu = document.getElementById("m-m");
  const mobileCanvas = document.getElementById("m-c");
  mobileCanvas.style.display = "block";
  mobileMenu.style.display = "flex";
}

const closeMobileMenu = () => {
  const mobileMenu = document.getElementById("m-m");
  const mobileCanvas = document.getElementById("m-c");
  mobileCanvas.style.display = "none";
  mobileMenu.style.display = "none";
}