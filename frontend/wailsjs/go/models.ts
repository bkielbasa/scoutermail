export namespace main {
	
	export class Account {
	    id: number;
	    name: string;
	    emailServer: string;
	    port: number;
	    useSSL: boolean;
	    username: string;
	    isActive: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Account(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.emailServer = source["emailServer"];
	        this.port = source["port"];
	        this.useSSL = source["useSSL"];
	        this.username = source["username"];
	        this.isActive = source["isActive"];
	    }
	}
	export class Attachment {
	    filename: string;
	    contentType: string;
	    size: number;
	    data: string;
	
	    static createFrom(source: any = {}) {
	        return new Attachment(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.filename = source["filename"];
	        this.contentType = source["contentType"];
	        this.size = source["size"];
	        this.data = source["data"];
	    }
	}
	export class Email {
	    id: number;
	    from: string;
	    subject: string;
	    date: string;
	    snippet: string;
	    attachmentCount: number;
	    read: boolean;
	
	    static createFrom(source: any = {}) {
	        return new Email(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.from = source["from"];
	        this.subject = source["subject"];
	        this.date = source["date"];
	        this.snippet = source["snippet"];
	        this.attachmentCount = source["attachmentCount"];
	        this.read = source["read"];
	    }
	}
	export class EmailContent {
	    headers: Record<string, string>;
	    textBody: string;
	    htmlBody: string;
	    attachments: Attachment[];
	
	    static createFrom(source: any = {}) {
	        return new EmailContent(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.headers = source["headers"];
	        this.textBody = source["textBody"];
	        this.htmlBody = source["htmlBody"];
	        this.attachments = this.convertValues(source["attachments"], Attachment);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class EmailPage {
	    emails: Email[];
	    totalCount: number;
	
	    static createFrom(source: any = {}) {
	        return new EmailPage(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.emails = this.convertValues(source["emails"], Email);
	        this.totalCount = source["totalCount"];
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

