class TomuLight {
	constructor(device) {
		this.usb = device;
	}

	static async get() {
		let device = await navigator.usb.requestDevice({
			filters: [{ vendorId: 0x1209, productId: 0x70b1 }],
		});
		await device.open();
		if (device.configuration === null) await device.selectConfiguration(1);
		await device.claimInterface(0);
		return new this(device);
	}

	async set(state) {
		await this.usb.controlTransferOut({
			requestType: "vendor",
			recipient: "device",
			request: 0,
			value: state,
			index: 0,
		});
	}

	off() {
		return this.set(0);
	}
	green() {
		return this.set(1);
	}
	red() {
		return this.set(2);
	}
	both() {
		return this.set(3);
	}
}

export default TomuLight;
