import React, { ChangeEvent, useEffect, useState } from "react";

export function Interval24Hours(props: {defaultValue?: number, onChange?: (k: number) => void}) {
    const defaultValue = props.defaultValue === undefined ? Math.floor(new Date().getTime() / (3600*24*1000)) : props.defaultValue;
    const startDate = new Date(0);
    startDate.setUTCSeconds(defaultValue * 24*3600);
    const [start, setStart] = useState(startDate);
    const endDate = new Date(0);
    endDate.setUTCSeconds((defaultValue+1) * 24*3600);
    const [end, setEnd] = useState(endDate);
    function setDay(day_: number) {
        const start_ = new Date(0);
        start_.setUTCSeconds(day_ * 24*3600);
        console.log(start_)
        setStart(start_);
        const end_ = new Date(0);
        end_.setUTCSeconds((day_+1) * 24*3600);
        setEnd(end_);
    }
    function handleOnChange(event: ChangeEvent) {
        const day_ = Number((event.target as HTMLInputElement).value);
        setDay(day_);
        if(props.onChange !== undefined) {
            props.onChange(day_);
        }
    }
    useEffect(() => {
        setDay(defaultValue);
    }, [])
    return (
        <>
            24 hours interval: <input type="number" defaultValue={defaultValue} onChange={handleOnChange}/> {" "}
            {start.toLocaleString()} - {end.toLocaleString()}
        </>
    );
}