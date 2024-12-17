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