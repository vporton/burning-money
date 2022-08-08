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
import { Listener } from 'history';
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
    const [totalBid, setTotalBid] = useState(0);
    const [totalReward, setTotalReward] = useState(0);
    useEffect(() => {
        setBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidAmount) && day !== null);
    }, [day, bidAmount])

    async function updateTotalBid() {
        const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
        const { chainId } = await provider.getNetwork();
        const addrs = (deployed as any)[CHAINS[chainId]];
        const token = new ethers.Contract(addrs.Token, tokenAbi, provider.getSigner(0));
        setTotalBid(await token.totalBids(BN.from(day)));
    }

    useEffect(() => {
        async function doIt() {
            await (window as any).ethereum.enable();
            const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
            const { chainId } = await provider.getNetwork();
            const addrs = (deployed as any)[CHAINS[chainId]];
            const token = new ethers.Contract(addrs.Token, tokenAbi, provider.getSigner(0));

            const listener = (_sender: any, _for: any, day_: number, _amount: any) => {
                if(day_ == day) {
                    updateTotalBid();
                }
            };
            token.off("Bid", listener);
            token.on("Bid", listener);
            updateTotalBid();
            const growthRate = Number(String(await token.growthRate())) / Math.pow(2, 64);
            const shift = Number(String(await token.shift())) / Math.pow(2, 64);
            setTotalReward(Math.floor(2**(-growthRate*day+shift)));
        }
        doIt();
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
            gasLimit: '300000',
        });
        // setInterval(async () => {
        //     console.log('BID:', day, await token.connect(provider.getSigner(0)).totalBids(BN.from(day)));
        // }, 1000);
    }

    return (
        <>
            <p>You mine CardToken by paying in Polkatod Glimmer or by a credit/debit card.</p>
            <p>You choose a future date for your bid. On or after this date you can withdraw
                CardToken in amount equal the share of you bid among all bids on this date
                multiplied by an exponent of time (for the day of bidding).</p>
            <p style={{color: 'red'}}>If your bid happens in past time, it won't happen, and our current policy is no refunds!</p>
            <p>Bid on: <Interval24Hours onChange={setDay}/></p>
            <p>Total bid on this time interval: {utils.formatEther(totalBid)} GLMR,
                competing for {totalReward/1e18} CT.</p>
            <br/>
            <Tabs>
                <TabList>
                    <Tab>Blockchain</Tab>
                    <Tab>Credit card or bank</Tab>
                </TabList>
                <TabPanel>
                    <p>Amount of GLMR you invest: {" "}
                        <input type="number" defaultValue={bidAmount} onChange={e => setBidAmount(e.target.value)} min="0.5" max="999999.99"
                        className={/^[0-9]+(\.[0-9]+)?$/.test(String(bidAmount)) ? "" : "error"}/></p>
                    <p><button onClick={bid} disabled={!bidButtonActive}>Bid</button></p>
                </TabPanel>
                <TabPanel>
                    <Card bidDay={day}/>
                </TabPanel>
            </Tabs>
        </>
    );
}
