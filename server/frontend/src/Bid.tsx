import Calendar from 'react-calendar'
import 'react-calendar/dist/Calendar.css';

export default function Bid() {
    const minDate = new Date();
    minDate.setDate(minDate.getDate() + 1);

    const bid = function() {
        
    };

    return <>
        <p>You bid for World Token by paying in Polkatod DOT.</p>
        <p>You choose a future date for your bid. On or after this date you can withdraw
            World Token in amount proportional to your bid and proportional to an
            exponent of time (for the moment you bid it for).</p>
        <p>Amount of DOT you transfer: <input type="number"/></p>
        <p>Bid date: <Calendar minDate={minDate} defaultValue={minDate}/></p>
        <p><button onClick={bid}>Bid</button></p>
    </>
}
