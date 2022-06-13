import { ethers } from 'hardhat';
import Calendar from 'react-calendar'
import 'react-calendar/dist/Calendar.css';
import deployed from "../../dist/deployed-addresses.json";
import { CHAINS } from './data';
import { abi as tokenAbi } from "../../artifacts/contracts/Token.sol/Token.json";
import { useEffect, useState } from 'react';
const { utils } = ethers;

export default function Bid() {
    const minDate = new Date();
    minDate.setDate(minDate.getDate() + 1);
    const [date, setDate] = useState(minDate);
    const [bidAmount, setBidAmount] = useState('');
    const [bidButtonActive, setBidButtonActive] = useState(false);
    useEffect(() => {
        setBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidAmount) && date !== null);
    }, [date, bidAmount])

    async function bid() {
        let provider = new ethers.providers.JsonRpcProvider();
        const { chainId } = await provider.getNetwork();
        const token = await ethers.getContractAt(tokenAbi, deployed[chainId].Token);
        const day = Math.floor(date / (24*3600))
        await token.bidOn(day, utils.parseEther(bidAmount));
    }
    
    return (
        <>
            <p>You bid for World Token by paying in Polkatod DOT.</p>
            <p>You choose a future date for your bid. On or after this date you can withdraw
                World Token in amount proportional to your bid and proportional to an
                exponent of time (for the moment you bid it for).</p>
            <p>Amount of DOT you transfer: <input type="number" defaultValue={bidAmount} onChange={e => setBidAmount(e.target.value)}/></p>
            <p>Bid date: <Calendar minDate={minDate} onChange={setDate}/></p>
            <p><button onClick={bid} disabled={!bidButtonActive}>Bid</button></p>
        </>
    );
}
