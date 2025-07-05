const kapchanState = {
  current_captcha: 0,
  hightlighted_msg: null,
};


const openAdminBoard = (e) => {
  let icon = e.querySelector('.dropdown');
  let container = e.parentElement.querySelector('.admin-board-info-container');

  if (icon.classList.contains("down")) {
    container.style.display = "flex";
    icon.classList.replace("down", "up");
    icon.children[0].setAttribute('d', "m280-400 200-200 200 200H280Z");
  } else {
    container.style.display = "none";
    icon.classList.replace("up", "down");
    icon.children[0].setAttribute('d', "M480-360 280-560h400L480-360Z");
  }
}

const openAdminBoardCreation = () => {
  const bc = document.getElementById("board-creation");

  if (bc) {
    bc.style.display = "flex";
  }
}

const closeAdminBoardCreation = () => {
  const bc = document.getElementById("board-creation");

  if (bc) {
    bc.style.display = "none";
  }
}

const openAdminBoardEdit = () => {
  const bc = document.getElementById("board-edit");

  if (bc) {
    bc.style.display = "flex";
  }
}

const closeAdminBoardEdit = () => {
  const bc = document.getElementById("board-edit");

  if (bc) {
    bc.style.display = "none";
  }
}

const banUserByPostId = () => {
  const bf = document.getElementById("ban-form");
  const data = new FormData(bf);

  let post_id = Number(data.get("post_id"));
  let ban_duration = Number(data.get("ban_duration"));
  let reason = data.get("reason");

  if (!post_id || !ban_duration) return;

  fetch(new Request("/ban-user-by-post/" + post_id, {
    method: "POST",
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      ban_duration_days: ban_duration,
      reason: reason
    })
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const showPost = (post_id) => {
  fetch(new Request("/post-details/" + post_id, {
    method: "GET",
  }))
  .then(res => res.json())
  .then(post_info => {
    location.replace(
      "/" + post_info.board_handle + "/thread/" + post_info.thread_id + "#p" + post_id
    );
  })
  .catch((error) => {
    console.log(error)
  });
}

const hintPost = (id) => {
  console.log(id); //TODO
}

const pinThread = (thread_id) => {
  fetch(new Request("/pin-thread/" + thread_id, {
    method: "GET",
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const unpinThread = (thread_id) => {
  fetch(new Request("/unpin-thread/" + thread_id, {
    method: "GET",
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const lockThread = (thread_id, lock_status) => {
  fetch(new Request("/lock-thread/" + thread_id, {
    method: "POST",
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      lock_status: lock_status
    })
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const deleteThread = (thread_id) => {
  fetch(new Request("/delete-thread/" + thread_id, {
    method: "POST",
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const deleteBoard = (board_id) => {
  fetch(new Request("/delete-board/" + board_id, {
    method: "POST",
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const deletePost = (post_id) => {
  fetch(new Request("/delete-post/" + post_id, {
    method: "POST",
  }))
  .then(res => {
    window.location.reload();
  })
  .catch((error) => {
    console.log(error)
  });
}

const showThreadMenu = (e) => {
  const dd = e.parentElement.querySelector('.thread-dropdown');

  if (dd.classList.contains("up")) {
    dd.style.display = "flex";
    dd.classList.replace("up", "down");
  } else {
    dd.style.display = "none";
    dd.classList.replace("down", "up");
  }
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

const openBanMenu = (post_id) => {
  const bm = document.getElementById("banMenu");
  bm.hidden = false;

  const bif = document.getElementById("post-id-field");
  bif.value=post_id;
}

const closeBanMenu = () => {
  const bm = document.getElementById("banMenu");
  bm.hidden = true;
}

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

const reloadImg = (img) => {
  let originalSrc = img.src;
  img.src = "/static/img/infinite-spinner.svg";

  setTimeout(() => {
    img.src = originalSrc;
  }, 3000);
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
  
  if (window.location.href.includes("#p")) {
    let post_id = window.location.href.match(/p(\d+)/g)[0];
    if (post_id) {
      const post = document.getElementById(post_id);
      if (post) {
        kapchanState.hightlighted_msg = post_id;
        post.style.border = "solid #FF4500"; 
      }
    }
  };
});

window.addEventListener("popstate", (event) => {
  if (kapchanState.hightlighted_msg) {
    const prev_post = document.getElementById(kapchanState.hightlighted_msg);

    if (prev_post) {
      prev_post.style.border = "";
    }
  }

  if (window.location.href.includes("#p")) {
    let post_id = window.location.href.match(/p(\d+)/g)[0];
    if (post_id) {
      const post = document.getElementById(post_id);
      if (post) {
        kapchanState.hightlighted_msg = post_id;
        post.style.border = "solid #FF4500"; 
      }
    }
  }
});