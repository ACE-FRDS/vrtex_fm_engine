import type { ClipboardItem, InspectorDetails } from '../types/clipboard'
import type { CollectionNode } from '../types/library'

export const sampleXml = `<?xml version="1.0" encoding="UTF-8"?>
<fmxmlsnippet type="FMObjectList">
  <Script includeInMenu="True" SiriShortcutVisible="False" runFullAccess="False" id="142" name="地方競馬現役馬JSON取込">
    <Step enable="True" id="141" name="Set Variable">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Value><Calculation><![CDATA["https://api.vertex.dev/nar/horses"]]></Calculation></Value>
      <Repetition><Calculation><![CDATA[1]]></Calculation></Repetition>
      <Name>$$url</Name>
    </Step>
    <Step enable="True" id="160" name="Insert from URL">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Calculation><![CDATA[$$url]]></Calculation>
      <Target>$$json</Target>
    </Step>
    <Step enable="True" id="141" name="Set Variable">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Value><Calculation><![CDATA[JSONListKeys ( $$json ; "" )]]></Calculation></Value>
      <Repetition><Calculation><![CDATA[1]]></Calculation></Repetition>
      <Name>$$keys</Name>
    </Step>
    <Step enable="True" id="71" name="Loop">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Restore state="False"></Restore>
      <FlushType value="Always"></FlushType>
    </Step>
    <Step enable="True" id="141" name="Set Variable">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Value><Calculation><![CDATA[$index + 1]]></Calculation></Value>
      <Repetition><Calculation><![CDATA[1]]></Calculation></Repetition>
      <Name>$index</Name>
    </Step>
    <Step enable="True" id="72" name="Exit Loop If">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
      <Calculation><![CDATA[$index ≥ ValueCount ( $$keys )]]></Calculation>
    </Step>
    <Step enable="True" id="73" name="End Loop">
      <DisableStepCollapsed state="False"></DisableStepCollapsed>
    </Step>
  </Script>
</fmxmlsnippet>`

const now = new Date()
const isoAt = (minutesAgo: number) => new Date(now.getTime() - minutesAgo * 60_000).toISOString()

export const sampleHistory: ClipboardItem[] = [
  {
    id: 'sample-script',
    name: '地方競馬現役馬JSON取込',
    format: 'XMSC',
    windowsFormat: 'Mac-XMSC',
    objectType: 'Script',
    xml: sampleXml,
    createdAt: isoAt(4),
    updatedAt: isoAt(4),
    lastUsedAt: isoAt(4),
    favorite: true,
    tags: ['Vertex', 'JSON', 'NAR'],
    notes: '地方競馬現役馬JSON取込用。Vertex Projectの中核インポートスクリプト。',
  },
  {
    id: 'sample-step',
    name: 'Insert from URL ステップ',
    format: 'XMSS',
    windowsFormat: 'Mac-XMSS',
    objectType: 'Step',
    xml: '<fmxmlsnippet><Step name="Insert from URL" /></fmxmlsnippet>',
    createdAt: isoAt(9),
    updatedAt: isoAt(9),
    lastUsedAt: isoAt(9),
    favorite: false,
    tags: ['JSON'],
    notes: '',
  },
  {
    id: 'sample-table',
    name: '出走馬テーブル',
    format: 'XMTB',
    windowsFormat: 'Mac-XMTB',
    objectType: 'Table',
    xml: '<fmxmlsnippet><BaseTable name="出走馬" /></fmxmlsnippet>',
    createdAt: isoAt(22),
    updatedAt: isoAt(22),
    lastUsedAt: isoAt(22),
    favorite: true,
    tags: ['NAR', 'Master'],
    notes: '',
  },
  {
    id: 'sample-field',
    name: '馬マスターフィールド',
    format: 'XMFD',
    windowsFormat: 'Mac-XMFD',
    objectType: 'Field',
    xml: '<fmxmlsnippet><Field name="horse_id" /></fmxmlsnippet>',
    createdAt: isoAt(58),
    updatedAt: isoAt(58),
    lastUsedAt: isoAt(58),
    favorite: false,
    tags: ['Master'],
    notes: '',
  },
  {
    id: 'sample-layout',
    name: 'メインレイアウトオブジェクト',
    format: 'XML2',
    windowsFormat: 'Mac-XML2',
    objectType: 'Layout',
    xml: '<fmxmlsnippet><Layout><Object type="Button" /></Layout></fmxmlsnippet>',
    createdAt: isoAt(1_500),
    updatedAt: isoAt(1_500),
    lastUsedAt: isoAt(1_500),
    favorite: false,
    tags: ['UI'],
    notes: '',
  },
]

export const sampleCollections: CollectionNode[] = [
  {
    id: 'vertex',
    name: 'Vertex Project',
    count: 14,
    children: [
      { id: 'jra-van', name: 'JRA-VAN', count: 4 },
      { id: 'nar', name: 'NAR', count: 5 },
      { id: 'import', name: 'Import Scripts', count: 3 },
      { id: 'ui-components', name: 'UI Components', count: 2 },
    ],
  },
  {
    id: 'medical',
    name: 'MedicalRecord',
    count: 8,
    children: [
      { id: 'analysis', name: 'Analysis Scripts', count: 3 },
      { id: 'temp', name: 'TEMP Tables', count: 5 },
    ],
  },
]

export const sampleInspector: InspectorDetails = {
  windowsFormat: 'Mac-XMSC',
  internalFormat: 'XMSC',
  objectType: 'Script',
  objects: '1 Script / 37 Steps',
  fileMakerVersion: '26.0',
  size: new TextEncoder().encode(sampleXml).byteLength,
  encoding: 'UTF-8',
  headerBytes: 4,
}
