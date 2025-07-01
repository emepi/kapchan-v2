const kapchanState = {
    current_captcha: 0,
};

const showPost = (id) => {
  console.log(id); //TODO
}

const hintPost = (id) => {
  console.log(id); //TODO
}

const scrollToBottom = () => {
  window.scrollTo(0, document.body.scrollHeight);
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
    const c = document.getElementById("captcha");
    const cc = document.getElementById("captcha-container");

    c.style.display = "grid";

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

    if (image_container.classList.contains("thumbnail")) {
        image_container.children[0].src = "/files/" + image_id;
        image_container.classList.replace("thumbnail", "image-container-large");
    } else {
        image_container.classList.replace("image-container-large", "thumbnail");
        image_container.children[0].src = "/thumbnails/" + image_id;
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

document.addEventListener("DOMContentLoaded", (event) => {
  let supportsTouch = 'ontouchstart' in window || navigator.msMaxTouchPoints;

  let mouseDown = false;
  let startX, scrollLeft;
  const slider = document.querySelector('.selector');

  const filePicker = document.getElementById('file-picker');
  const fileChosen = document.getElementById('file-chosen');

  if (filePicker && fileChosen) {
    filePicker.addEventListener('change', () => {
        fileChosen.textContent = filePicker.files[0].name;
    });
  }

  const startDragging = (e) => {
    mouseDown = true;
    startX = e.pageX - slider.offsetLeft;
    scrollLeft = slider.scrollLeft;
  }
  
  const stopDragging = (e) => {
    mouseDown = false;
  }
  
  const move = (e) => {
    e.preventDefault();
    if(!mouseDown) { return; }
    const x = e.pageX - slider.offsetLeft;
    const scroll = x - startX;
    slider.scrollLeft = scrollLeft - scroll;
  }

  if (!supportsTouch && slider) {
    slider.addEventListener('mousemove', move, false);
    slider.addEventListener('mousedown', startDragging, false);
    slider.addEventListener('mouseup', stopDragging, false);
    slider.addEventListener('mouseleave', stopDragging, false);
  }

  document.querySelectorAll(".msg-lbl").forEach((msg) => {
    let text = msg.textContent;

    //Regex strings
	let find = [
        /&/g,
		/<(.*?)>/g,
        /^(?=>[^>])>([^\r\n]+)/gm,
        />>(\d+)/g,
		/\[spoiler\](.*?)\[\/spoiler\]/g,
        /(([https?|ftp]+:\/\/)([^\s/?\.#-]+\.?)+(\/[^\s]*)?)/gi,
	];

	//Regex string replacements
	let replace = [
        '&amp;',
		'&lt;$1&gt;',
        '<span class="implying">&gt;$1</span>',
        '<span class="backlink" onClick="showPost($1)" onmouseenter="hintPost($1)">&gt;&gt;$1</span>',
		'<span class="spoiler">$1</span>',
        '<a class="link" href="$1">$1</a>',
	];

    for (let i =0; i < find.length; i++) {
	  text = text.replace(find[i], replace[i]);
	}

    msg.innerHTML = text;
  })
});