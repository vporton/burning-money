// import { ethers } from 'hardhat';
import { ethers } from 'ethers';
import Calendar from 'react-calendar'
import 'react-calendar/dist/Calendar.css';
import deployed from "../../dist/deployed-addresses.json";
import { CHAINS } from './data';
import { abi as tokenAbi } from "../../artifacts/contracts/Token.sol/Token.json";
import { abi as erc20Abi } from "@openzeppelin/contracts/build/contracts/ERC20.json";
import { useEffect, useState } from 'react';
import { Tab, TabList, TabPanel, Tabs } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';
import { backendUrlPrefix } from '../config';
const { utils, BigNumber: BN } = ethers;

export default function Bid() {
    const minDate = new Date();
    minDate.setDate(minDate.getDate() + 1);
    const [date, setDate] = useState(minDate);
    const [bidAmount, setBidAmount] = useState('');
    const [bidButtonActive, setBidButtonActive] = useState(false);
    const [bidCCAmount, setCCAmount] = useState('');
    const [ccBidButtonActive, setCCBidButtonActive] = useState(false);
    useEffect(() => {
        setBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidAmount) && date !== null);
    }, [date, bidAmount])
    useEffect(() => {
        setCCBidButtonActive(/^[0-9]+(\.[0-9]+)?/.test(bidCCAmount) && date !== null);
    }, [date, bidCCAmount])

    async function bid() {
        await (window as any).ethereum.enable();
        const provider = new ethers.providers.Web3Provider((window as any).ethereum, "any");
        const { chainId } = await provider.getNetwork();
        if(!CHAINS[chainId] || !deployed[CHAINS[chainId]]) {
            alert("This chain is not supported"); // TODO
            return;
        }
        const token = new ethers.Contract(deployed[CHAINS[chainId]].Token, tokenAbi);
        const collateral = new ethers.Contract(deployed[CHAINS[chainId]].collateral, erc20Abi);
        const day = Math.floor(date.getTime() / 1000 / (24*3600));
        // const estimation = await token.estimateGas.bidOn(day, utils.parseEther(bidAmount)); // TODO
        const allowance = await collateral.connect(provider.getSigner(0)).allowance(await (await provider.getSigner(0)).getAddress(), token.address);
        if(allowance.lt(utils.parseEther(bidAmount))) {
            await collateral.connect(provider.getSigner(0)).approve(token.address, utils.parseEther(bidAmount));
        }
        await token.connect(provider.getSigner(0)).bidOn(day, utils.parseEther(bidAmount), {
            // gasLimit: String(estimation.mul(BN.from(1.3))), // TODO
            gasLimit: '200000',
        });
    }

    async function ccBid() {
        console.log("open: ", backendUrlPrefix + "/create-stripe-checkout");
        open(backendUrlPrefix + "/create-stripe-checkout?fiat_amount=" + bidCCAmount, '_self');
    }

    return (
        <>
            <p>You mine World Token by paying in Polkatod Glimmer.</p>
            <p>You choose a future date for your bid. On or after this date you can withdraw
                World Token in amount equal the share of you bid among all bids on this date
                multiplied by an exponent of time (for the day of bidding).</p>
            <p>Bid date: <Calendar minDate={minDate} onChange={setDate}/></p>
            <Tabs>
                <TabList>
                    <Tab>Blockchain</Tab>
                    <Tab>Credit card or bank</Tab>
                </TabList>
                <TabPanel>
                    <p>Amount of GLMR you pay:
                        <input type="number" defaultValue={bidAmount} onChange={e => setBidAmount(e.target.value)} min="0.5" max="999999.99"/></p>
                    <p><button onClick={bid} disabled={!bidButtonActive}>Bid</button></p>
                </TabPanel>
                <TabPanel>
                    <p>Amount in USD you pay: <input type="number" onChange={e => setCCAmount(e.target.value)}/></p>
                    <p><button onClick={ccBid} disabled={!ccBidButtonActive}>Bid</button></p>
                </TabPanel>
            </Tabs>
        </>
    );
}
