import { v4 as uuidv4 } from 'uuid';

interface DocumentChunk {
    id: string;
    text: string;
    position: number;
}

interface ProcessedDocument {
    metadata: {
        id: string;
        title: string;
        content_type: string;
        source_url: string;
        timestamp: bigint;
        total_chunks: number;
        size: number;
        is_embedded: boolean;
    };
    chunks: DocumentChunk[];
}

export class DocumentProcessor {
    private static readonly MAX_CHUNK_SIZE = 1000; // Characters per chunk
    private static readonly MIN_CHUNK_SIZE = 100; // Minimum characters per chunk

    /**
     * Process a document and prepare it for upload
     * @param content The document content
     * @param title The document title
     * @param sourceUrl Optional source URL
     * @returns Processed document ready for upload
     */
    public static processDocument(
        content: string,
        title: string,
        sourceUrl?: string
    ): ProcessedDocument {
        // Generate document ID
        const documentId = uuidv4();
        
        // Split content into chunks
        const chunks = this.splitIntoChunks(content);
        
        // Create metadata
        const metadata = {
            id: documentId,
            title,
            content_type: 'text/plain',
            source_url: sourceUrl || '',
            timestamp: BigInt(Date.now() * 1_000_000), // Convert to nanoseconds
            total_chunks: chunks.length,
            size: content.length,
            is_embedded: false
        };

        return {
            metadata,
            chunks
        };
    }

    /**
     * Split text into semantic chunks
     * @param text The text to split
     * @returns Array of chunks with IDs and positions
     */
    private static splitIntoChunks(text: string): DocumentChunk[] {
        const chunks: DocumentChunk[] = [];
        let currentPosition = 0;
        let currentChunk = '';
        
        // Split text into sentences (basic implementation)
        const sentences = text.match(/[^.!?]+[.!?]+/g) || [text];
        
        for (const sentence of sentences) {
            // If adding this sentence would exceed max chunk size, save current chunk
            if (currentChunk.length + sentence.length > this.MAX_CHUNK_SIZE && currentChunk.length >= this.MIN_CHUNK_SIZE) {
                chunks.push({
                    id: uuidv4(),
                    text: currentChunk.trim(),
                    position: currentPosition++
                });
                currentChunk = '';
            }
            
            currentChunk += sentence;
        }
        
        // Add the last chunk if it's not empty
        if (currentChunk.trim()) {
            chunks.push({
                id: uuidv4(),
                text: currentChunk.trim(),
                position: currentPosition
            });
        }
        
        return chunks;
    }

    /**
     * Prepare a document for upload and embedding proposal
     * @param processedDoc The processed document
     * @param collectionId The collection ID
     * @returns Operations to perform
     */
    public static prepareForUpload(processedDoc: ProcessedDocument, collectionId: string) {
        return {
            // First, add the document
            addDocument: {
                collectionId,
                title: processedDoc.metadata.title,
                content: processedDoc.chunks.map(chunk => chunk.text).join('\n\n')
            },
            // Then create an embedding proposal
            createProposal: {
                collectionId,
                documentId: processedDoc.metadata.id,
                description: `Embed document: ${processedDoc.metadata.title}`
            }
        };
    }
} 