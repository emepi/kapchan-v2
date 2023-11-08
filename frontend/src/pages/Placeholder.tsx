import boomer from '/src/assets/boomer.jpg'

function Placeholder() {
    return (
        <div class="placeholder-page">
          <section>
            <h2>Welcome to avaruuskapakka</h2>
            <p>We are in development!</p>
          </section>
          <img src={boomer} alt="Boomer judging the state of kapchan" />
        </div>
    )
}

export default Placeholder