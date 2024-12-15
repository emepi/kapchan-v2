const logout = (event) => {
    fetch(new Request("/logout", {
            method: "POST",
        })
    )
    .then(() => location.reload());
};