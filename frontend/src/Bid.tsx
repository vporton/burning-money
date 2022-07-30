// import { ethers } from 'hardhat';
import { ethers } from 'ethers';
// import Calendar from 'react-calendar'
// import 'react-calendar/dist/Calendar.css';
import deployed from "./deployed-addresses.json";
import { CHAINS } from './data';
import Token from "./Token.json";
import ERC20 from "@openzeppelin/contracts/build/contracts/ERC20.json";
import { useEffect, useState } from 'react';
import { Tab, TabList, TabPanel, Tabs } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';
import { backendUrlPrefix } from './config';
import Card from './Card';
import React from 'react';
import { Interval24Hours } from './components/Interval24Hours';
const { utils, BigNumber: BN } = ethers;
const tokenAbi = Token.abi;
const erc20Abi = ERC20.abi;

export default function Bid() {
    // const minDate = new Date();
    // minDate.setUTCDate(minDate.getUTCDate() + 1);
    // minDate.setUTCHours(0);
    // minDate.setUTCMinutes(0);
    // minDate.setUTCSeconds(0);
    // minDate.setUTCMilliseconds(0);
    const [day, setDay] = useState(Math.floor(new Date().getTime() / (24*3600*1000)));
    const [bidAmount, setBidAmount] = useState('');
    const [bidButtonActive, setBidButtonActive] = useState(false);
    const [bidCCAmount, setCCAmount] = useState('');
    const [ccBidButtonActive, setCCBidButtonActive] = useState(false);
    useEffect(() => {
        setBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidAmount) && day !== null);
    }, [day, bidAmount])
    useEffect(() => {
        setCCBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidCCAmount) && day !== null);
    }, [day, bidCCAmount])

    useEffect(() => {
        console.log(`Switching to ${day}`);
    }, [day]);

    async function bid() {
        await (window as any).ethereum.enable();
        const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
        const { chainId } = await provider.getNetwork();
        if(!CHAINS[chainId] || !(CHAINS[chainId] in deployed)) {
            alert("This chain is not supported"); // TODO
            return;
        }
        const addrs = (deployed as any)[CHAINS[chainId]];
        const token = new ethers.Contract(addrs.Token, tokenAbi);
        console.log(`Bidding ${utils.parseEther(bidAmount)} on ${day}`);
        // const estimation = await token.estimateGas.bidOn(day, utils.parseEther(bidAmount)); // TODO
        await token.connect(provider.getSigner(0)).bidOn(day, await provider.getSigner(0).getAddress(), {
            value: utils.parseEther(bidAmount),
            // gasLimit: String(estimation.mul(BN.from(1.3))), // TODO
            gasLimit: '200000',
        });
        // setInterval(async () => {
        //     console.log('BID:', day, await token.connect(provider.getSigner(0)).totalBids(BN.from(day)));
        // }, 1000);
    }

    async function ccBid() {
        // TODO: fiat_amount in 0.50 .. 999999.99
        window.open(backendUrlPrefix + "/create-stripe-checkout?fiat_amount=" + bidCCAmount, '_self');
    }

    return (
        <>
            <p>You mine CardToken by paying in Polkatod Glimmer.</p>
            <p>You choose a future date for your bid. On or after this date you can withdraw
                CardToken in amount equal the share of you bid among all bids on this date
                multiplied by an exponent of time (for the day of bidding).</p>
            <p>Bid date:</p>
            <Interval24Hours onChange={setDay}/>
            <br/>
            <Tabs>
                <TabList>
                    <Tab>Blockchain</Tab>
                    <Tab>Credit card or bank</Tab>
                </TabList>
                <TabPanel>
                    <p>Amount of GLMR you invest: {" "}
                        <input type="number" defaultValue={bidAmount} onChange={e => setBidAmount(e.target.value)} min="0.5" max="999999.99"/></p>
                    <p><button onClick={bid} disabled={!bidButtonActive}>Bid</button></p>
                </TabPanel>
                <TabPanel>
                    <Card bidDay={day}/>
                </TabPanel>
            </Tabs>
        </>
    );
}
